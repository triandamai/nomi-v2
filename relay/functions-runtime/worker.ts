// One project environment's functions, inside a worker that can reach the network and read its
// own build, nothing else. The runtime sends `init` once (the files to load and `ctx.env`), then
// one `call` per request; each call answers with a `result`.
/// <reference lib="deno.worker" />

type Handler = (req: Request, ctx: unknown) => Response | Promise<Response>;
type ContextFactory = (req: Request, env: Record<string, string | undefined>) => unknown;

const SDK_FUNCTIONS = 'npm:nomi-relay@0.1.2/functions';

let handlers: Record<string, Handler> = {};
let env: Record<string, string> = {};
let createContext: ContextFactory | null = null;

function errorText(error: unknown): string {
  return error instanceof Error ? (error.stack ?? error.message) : String(error);
}

async function init(entries: Record<string, string>, givenEnv: Record<string, string>): Promise<void> {
  env = givenEnv;
  const loaded: Record<string, Handler> = {};
  for (const [name, url] of Object.entries(entries)) {
    let module: { default?: unknown };
    try {
      module = await import(url);
    } catch (error) {
      self.postMessage({ type: 'failed', error: `${name} failed to load: ${errorText(error)}` });
      return;
    }
    if (typeof module.default !== 'function') {
      self.postMessage({ type: 'failed', error: `${name}'s default export isn't a function: export default defineFunction(async (req, ctx) => …).` });
      return;
    }
    loaded[name] = module.default as Handler;
  }
  try {
    createContext = ((await import(SDK_FUNCTIONS)) as { createFunctionContext: ContextFactory }).createFunctionContext;
  } catch (error) {
    self.postMessage({ type: 'failed', error: `nomi-relay couldn't be loaded: ${errorText(error)}` });
    return;
  }
  handlers = loaded;
  self.postMessage({ type: 'ready' });
}

interface Call {
  id: number;
  name: string;
  method: string;
  url: string;
  headers: [string, string][];
  body: ArrayBuffer | null;
}

async function call(message: Call): Promise<void> {
  let response: Response;
  try {
    const request = new Request(message.url, { method: message.method, headers: message.headers, body: message.body });
    response = await handlers[message.name](request, createContext!(request, env));
    if (!(response instanceof Response)) throw new Error(`${message.name} must return a Response`);
  } catch (error) {
    console.error(`${message.name}: ${errorText(error)}`);
    response = Response.json({ code: 'FUNCTION_ERROR', message: `${message.name} threw: ${error instanceof Error ? error.message : String(error)}` }, { status: 500 });
  }
  const body = response.body ? await response.arrayBuffer() : null;
  self.postMessage({ type: 'result', id: message.id, status: response.status, headers: [...response.headers], body }, body ? [body] : []);
}

self.onmessage = (event: MessageEvent) => {
  const message = event.data;
  if (message.type === 'init') init(message.entries, message.env);
  else if (message.type === 'call') call(message);
};
