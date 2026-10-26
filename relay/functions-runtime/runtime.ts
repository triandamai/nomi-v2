// Nomi Relay's functions runtime: runs each project environment's `nomi/functions/` in its own
// Deno worker, which can reach the network and read its own build and nothing else (no process
// environment, files, subprocesses or FFI). Only Relay's gateway calls it, with the shared secret.
//
//   GET /health
//   PUT /builds/<project>/<env>/<version>  { files: [{ path, content }], env: { NAME: value } }
//   *   /run/<project>/<env>/<version>/<name>[/rest]   (x-nomi-claims, x-relay-timeout)

/** The `nomi-relay` package every build's `nomi-relay` imports resolve to. */
export const DEFAULT_SDK = 'npm:nomi-relay@0.1.2';
/** A call's timeout when the gateway doesn't say. */
const DEFAULT_TIMEOUT_SECONDS = 30;
const MAX_TIMEOUT_SECONDS = 60;
/** How long a type check may take. */
const CHECK_TIMEOUT_MS = 120_000;
/** Request and response bodies stop here. */
export const MAX_BODY_BYTES = 6 * 1024 * 1024;
const NAME = /^[a-z0-9][a-z0-9-]{0,62}$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const ENVIRONMENTS = new Set(['dev', 'production']);
const FUNCTIONS_PREFIX = 'nomi/functions/';
const SOURCE_EXTENSIONS = ['.ts', '.js', '.mts', '.mjs', '.json'];
const IDLE_CHECK_MS = 60_000;

export interface RuntimeOptions {
  secret: string;
  dataDir: string;
  /** How long a build's files may take to load in its worker (a loop on import fails here). */
  initTimeoutMs: number;
  /** A worker unused this long is stopped; its next call starts it again. */
  idleMs: number;
  sdk?: string;
}

interface SourceFile {
  path: string;
  content: string;
}

/** A path inside the functions folder, without its `nomi/functions/` prefix; `null` when it could leave the folder. */
export function cleanPath(path: string): string | null {
  const relative = path.startsWith(FUNCTIONS_PREFIX) ? path.slice(FUNCTIONS_PREFIX.length) : path;
  if (!relative || relative.startsWith('/') || relative.includes('\\') || /[\u0000-\u001f]/.test(relative)) return null;
  const segments = relative.split('/');
  if (segments.some((s) => s === '' || s === '.' || s === '..')) return null;
  if (!SOURCE_EXTENSIONS.some((ext) => relative.endsWith(ext))) return null;
  return relative;
}

/** Points `nomi-relay` and `nomi-relay/functions` imports at the pinned package. */
export function rewriteImports(source: string, sdk: string): string {
  return source.replace(/(\bfrom\s*|\bimport\s*\(\s*|\bimport\s+)(['"])nomi-relay(\/functions)?\2/g, (_m, lead, quote, sub) => `${lead}${quote}${sdk}${sub ?? ''}${quote}`);
}

function same(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

function json(status: number, body: unknown, headers: HeadersInit = {}): Response {
  return Response.json(body, { status, headers });
}

function refuse(status: number, code: string, message: string): Response {
  return json(status, { code, message });
}

/** `deno check` output, with ANSI colours and the build's own folder taken out. */
function readable(text: string, dir: string): string {
  // deno-lint-ignore no-control-regex
  return text.replace(/\u001b\[[0-9;]*m/g, '').replaceAll(`file://${dir}/`, FUNCTIONS_PREFIX).replaceAll(`${dir}/`, FUNCTIONS_PREFIX).trim();
}

interface Pending {
  resolve: (message: WorkerResult) => void;
}

interface WorkerResult {
  type: 'result';
  id: number;
  status: number;
  headers: [string, string][];
  body: ArrayBuffer | null;
}

/** One project environment's current build: its files on disk and, while in use, its worker. */
class Build {
  worker: Worker | null = null;
  lastUsed = Date.now();
  private nextId = 1;
  private pending = new Map<number, Pending>();

  constructor(
    readonly version: number,
    readonly dir: string,
    readonly entries: Record<string, string>,
    readonly env: Record<string, string>,
    private readonly initTimeoutMs: number,
  ) {}

  /** Starts the worker and loads every function; answers what went wrong, or null. */
  async start(): Promise<string | null> {
    this.stop();
    const worker = new Worker(new URL('./worker.ts', import.meta.url).href, {
      type: 'module',
      deno: {
        permissions: { net: true, read: [this.dir], env: false, run: false, write: false, ffi: false, sys: false, import: true },
      },
    } as WorkerOptions);
    this.worker = worker;
    const loaded = new Promise<string | null>((resolve) => {
      const timer = setTimeout(() => resolve(`a function didn't finish loading in ${this.initTimeoutMs / 1000} seconds (keep work at the top of a file light)`), this.initTimeoutMs);
      worker.onmessage = (event: MessageEvent) => {
        const message = event.data;
        if (message.type === 'ready') {
          clearTimeout(timer);
          worker.onmessage = (e: MessageEvent) => this.onResult(e.data);
          resolve(null);
        } else if (message.type === 'failed') {
          clearTimeout(timer);
          resolve(message.error);
        }
      };
      worker.onerror = (event: ErrorEvent) => {
        event.preventDefault();
        clearTimeout(timer);
        resolve(`a function failed to load: ${event.message}`);
      };
    });
    worker.postMessage({ type: 'init', entries: this.entries, env: this.env });
    const failure = await loaded;
    if (failure) this.stop();
    else worker.onerror = (event: ErrorEvent) => {
      event.preventDefault();
      this.stop();
    };
    return failure;
  }

  stop(): void {
    this.worker?.terminate();
    this.worker = null;
    for (const [id, waiting] of this.pending) {
      waiting.resolve({ type: 'result', id, status: 502, headers: [['content-type', 'application/json']], body: new TextEncoder().encode(JSON.stringify({ code: 'FUNCTION_CRASHED', message: 'The function stopped before answering.' })).buffer as ArrayBuffer });
    }
    this.pending.clear();
  }

  private onResult(message: WorkerResult): void {
    const waiting = this.pending.get(message.id);
    if (!waiting) return;
    this.pending.delete(message.id);
    waiting.resolve(message);
  }

  async call(name: string, request: Request, publicUrl: string, timeoutSeconds: number): Promise<Response> {
    this.lastUsed = Date.now();
    if (!this.worker) {
      const failure = await this.start();
      if (failure) return refuse(500, 'FUNCTION_FAILED', failure);
    }
    const body = request.body ? await readLimited(request) : null;
    if (body === undefined) return refuse(413, 'TOO_LARGE', `A function's request body can be up to ${MAX_BODY_BYTES / 1024 / 1024} MB.`);
    const id = this.nextId++;
    const answer = new Promise<WorkerResult>((resolve) => this.pending.set(id, { resolve }));
    const headers = [...request.headers].filter(([k]) => k !== 'x-relay-runtime' && k !== 'x-relay-timeout');
    this.worker!.postMessage({ type: 'call', id, name, method: request.method, url: publicUrl, headers, body }, body ? [body] : []);
    let timer: number | undefined;
    const timedOut = new Promise<null>((resolve) => (timer = setTimeout(() => resolve(null), timeoutSeconds * 1000)));
    const result = await Promise.race([answer, timedOut]);
    clearTimeout(timer);
    this.lastUsed = Date.now();
    if (!result) {
      // A worker stuck in one call may be stuck for good: start it again for the next one.
      this.pending.delete(id);
      this.stop();
      return refuse(504, 'FUNCTION_TIMEOUT', `${name} didn't answer within ${timeoutSeconds} seconds.`);
    }
    return new Response(result.body, { status: result.status, headers: result.headers });
  }
}

/** The body, or `undefined` when it's over the limit. */
async function readLimited(request: Request): Promise<ArrayBuffer | undefined> {
  const reader = request.body!.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.byteLength;
    if (size > MAX_BODY_BYTES) {
      await reader.cancel();
      return undefined;
    }
    chunks.push(value);
  }
  const out = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return out.buffer;
}

export function createRuntime(options: RuntimeOptions) {
  const sdk = options.sdk ?? DEFAULT_SDK;
  const builds = new Map<string, Build>();
  const idle = setInterval(() => {
    const now = Date.now();
    for (const build of builds.values()) if (build.worker && now - build.lastUsed > options.idleMs) build.stop();
  }, IDLE_CHECK_MS);

  async function deploy(project: string, env: string, version: number, request: Request): Promise<Response> {
    let input: { files?: SourceFile[]; env?: Record<string, string> };
    try {
      input = await request.json();
    } catch {
      return refuse(400, 'BAD_BODY', 'Send { files, env }.');
    }
    const files: SourceFile[] = [];
    for (const file of input.files ?? []) {
      const path = cleanPath(file.path ?? '');
      if (!path) return refuse(400, 'BAD_PATH', `${file.path} isn't a source file inside nomi/functions/.`);
      files.push({ path, content: String(file.content ?? '') });
    }
    const entries: Record<string, string> = {};
    for (const file of files) {
      if (file.path.includes('/') || file.path.startsWith('_') || !/\.(ts|js|mts|mjs)$/.test(file.path)) continue;
      const name = file.path.replace(/\.[a-z]+$/, '');
      if (!NAME.test(name)) return refuse(400, 'BAD_NAME', `${file.path}: a function's file name is lowercase letters, digits and -, like place-order.ts.`);
      entries[name] = file.path;
    }
    if (Object.keys(entries).length === 0) return json(200, { ok: false, error: 'nomi/functions/ has no functions: write one file per function, like nomi/functions/place-order.ts.' });

    const dir = `${options.dataDir}/${project}/${env}/${version}`;
    await Deno.remove(dir, { recursive: true }).catch(() => {});
    for (const file of files) {
      const target = `${dir}/${file.path}`;
      await Deno.mkdir(target.slice(0, target.lastIndexOf('/')), { recursive: true });
      await Deno.writeTextFile(target, rewriteImports(file.content, sdk));
    }
    const fileUrls = Object.fromEntries(Object.entries(entries).map(([name, path]) => [name, `file://${dir}/${path}`]));

    const checked = await check(dir, Object.values(entries));
    if (checked) {
      await Deno.remove(dir, { recursive: true }).catch(() => {});
      return json(200, { ok: false, error: checked });
    }
    const build = new Build(version, dir, fileUrls, input.env ?? {}, options.initTimeoutMs);
    const failure = await build.start();
    if (failure) {
      await Deno.remove(dir, { recursive: true }).catch(() => {});
      return json(200, { ok: false, error: readable(failure, dir) });
    }
    const key = `${project}/${env}`;
    const previous = builds.get(key);
    builds.set(key, build);
    if (previous && previous.dir !== dir) {
      previous.stop();
      await Deno.remove(previous.dir, { recursive: true }).catch(() => {});
    }
    return json(200, { ok: true, functions: Object.keys(entries).sort() });
  }

  /** Type-checks the entry files; answers the errors, or null. */
  async function check(dir: string, entries: string[]): Promise<string | null> {
    const command = new Deno.Command(Deno.execPath(), {
      args: ['check', '--quiet', ...entries],
      cwd: dir,
      stdout: 'piped',
      stderr: 'piped',
      env: { NO_COLOR: '1' },
    });
    const child = command.spawn();
    const timer = setTimeout(() => child.kill(), CHECK_TIMEOUT_MS);
    const output = await child.output();
    clearTimeout(timer);
    if (output.success) return null;
    const text = new TextDecoder().decode(output.stderr) + new TextDecoder().decode(output.stdout);
    return readable(text, dir) || 'the type check failed';
  }

  async function run(project: string, env: string, version: number, rest: string[], request: Request): Promise<Response> {
    const build = builds.get(`${project}/${env}`);
    if (!build || build.version !== version) return refuse(412, 'NOT_LOADED', 'That build is not loaded here.');
    const [name, ...tail] = rest;
    if (!name || !build.entries[name]) return refuse(404, 'NO_FUNCTION', `There's no function called ${name}.`);
    const timeout = Math.min(MAX_TIMEOUT_SECONDS, Math.max(1, Number(request.headers.get('x-relay-timeout')) || DEFAULT_TIMEOUT_SECONDS));
    const base = (build.env.NOMI_URL ?? 'http://localhost').replace(/\/+$/, '');
    const query = new URL(request.url).search;
    const publicUrl = `${base}/functions/${[name, ...tail].join('/')}${query}`;
    return build.call(name, request, publicUrl, timeout);
  }

  async function handle(request: Request): Promise<Response> {
    if (!same(request.headers.get('x-relay-runtime') ?? '', options.secret)) return refuse(401, 'UNAUTHORIZED', 'Only Relay calls the functions runtime.');
    const segments = new URL(request.url).pathname.split('/').filter(Boolean).map(decodeURIComponent);
    if (segments[0] === 'health' && segments.length === 1) return json(200, { ok: true });
    const [kind, project, env, versionText, ...rest] = segments;
    const version = Number(versionText);
    if ((kind === 'builds' || kind === 'run') && (!UUID.test(project ?? '') || !ENVIRONMENTS.has(env ?? '') || !Number.isInteger(version) || version < 1)) {
      return refuse(400, 'BAD_PATH', 'Expected /<builds|run>/<project id>/<dev|production>/<version>.');
    }
    if (kind === 'builds' && request.method === 'PUT' && rest.length === 0) return deploy(project, env, version, request);
    if (kind === 'run') return run(project, env, version, rest, request);
    return refuse(404, 'NOT_FOUND', 'Nothing here.');
  }

  async function close(): Promise<void> {
    clearInterval(idle);
    for (const build of builds.values()) build.stop();
    builds.clear();
    await Deno.remove(options.dataDir, { recursive: true }).catch(() => {});
  }

  return { handle, close };
}
