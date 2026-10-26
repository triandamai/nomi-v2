// The functions runtime as the gateway uses it: builds are checked and loaded, calls run in the
// project's own worker with the caller's claims and the project's secrets, and nothing outside
// the build or the worker's sandbox is reachable.
import { assert, assertEquals, assertStringIncludes } from 'jsr:@std/assert@1';
import { cleanPath, createRuntime, rewriteImports } from './runtime.ts';

const SECRET = 'runtime-secret';
const PROJECT = '7c9e6679-7425-40de-944b-e07fc1f90ae7';
const ENV = {
  NOMI_URL: 'https://todo-dev.relay.test',
  NOMI_ANON_KEY: 'anon',
  NOMI_SERVICE_KEY: 'service',
  STRIPE_SECRET: 'sk_test_123',
};

async function runtime(options: { initTimeoutMs?: number } = {}) {
  const dataDir = await Deno.makeTempDir();
  return createRuntime({ secret: SECRET, dataDir, initTimeoutMs: options.initTimeoutMs ?? 20_000, idleMs: 60_000 });
}

function call(rt: Awaited<ReturnType<typeof runtime>>, method: string, path: string, init: { body?: unknown; headers?: Record<string, string> } = {}) {
  const headers = new Headers({ 'x-relay-runtime': SECRET, ...init.headers });
  let body: BodyInit | undefined;
  if (init.body !== undefined) {
    body = typeof init.body === 'string' ? init.body : JSON.stringify(init.body);
    if (!headers.has('content-type')) headers.set('content-type', 'application/json');
  }
  return rt.handle(new Request(`http://runtime${path}`, { method, headers, body }));
}

function claims(value: Record<string, unknown>): string {
  return btoa(JSON.stringify(value)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

const HELLO = `import { defineFunction } from 'nomi-relay/functions';
import { greeting } from './_shared/words.ts';
export default defineFunction(async (req, ctx) => {
  const body = req.method === 'POST' ? await req.json() : null;
  return Response.json({ greeting: greeting(ctx.user?.id ?? 'stranger'), role: ctx.claims.role, stripe: ctx.env.STRIPE_SECRET, body, path: new URL(req.url).pathname });
});
`;
const WORDS = `export const greeting = (who: string): string => 'hello ' + who;\n`;

async function deployHello(rt: Awaited<ReturnType<typeof runtime>>, version = 1) {
  const answer = await call(rt, 'PUT', `/builds/${PROJECT}/dev/${version}`, {
    body: { files: [{ path: 'nomi/functions/hello.ts', content: HELLO }, { path: 'nomi/functions/_shared/words.ts', content: WORDS }], env: ENV },
  });
  return { status: answer.status, body: await answer.json() };
}

Deno.test('nothing answers without the runtime secret', async () => {
  const rt = await runtime();
  const answer = await rt.handle(new Request('http://runtime/health'));
  assertEquals(answer.status, 401);
  await answer.body?.cancel();
  const ok = await call(rt, 'GET', '/health');
  assertEquals(ok.status, 200);
  await ok.body?.cancel();
  await rt.close();
});

Deno.test('a build is checked, loaded and called with the claims and secrets', async () => {
  const rt = await runtime();
  const deployed = await deployHello(rt);
  assertEquals(deployed, { status: 200, body: { ok: true, functions: ['hello'] } });

  const answer = await call(rt, 'POST', `/run/${PROJECT}/dev/1/hello/sub/path?x=1`, {
    body: { cart: 3 },
    headers: { 'x-nomi-claims': claims({ role: 'authenticated', sub: 'user-1' }), authorization: 'Bearer user-token', 'x-relay-timeout': '5' },
  });
  assertEquals(answer.status, 200);
  assertEquals(await answer.json(), { greeting: 'hello user-1', role: 'authenticated', stripe: 'sk_test_123', body: { cart: 3 }, path: '/functions/hello/sub/path' });
  await rt.close();
});

Deno.test('a call for a build that is not loaded asks for it', async () => {
  const rt = await runtime();
  const answer = await call(rt, 'GET', `/run/${PROJECT}/dev/1/hello`);
  assertEquals(answer.status, 412);
  assertEquals((await answer.json()).code, 'NOT_LOADED');
  await deployHello(rt, 2);
  const stale = await call(rt, 'GET', `/run/${PROJECT}/dev/1/hello`);
  assertEquals(stale.status, 412, 'an older version is not served');
  await stale.body?.cancel();
  const missing = await call(rt, 'GET', `/run/${PROJECT}/dev/2/nope`);
  assertEquals(missing.status, 404);
  await missing.body?.cancel();
  await rt.close();
});

Deno.test('a type error fails the deploy with the file and line, and the running build stays', async () => {
  const rt = await runtime();
  await deployHello(rt);
  const broken = await call(rt, 'PUT', `/builds/${PROJECT}/dev/2`, {
    body: { files: [{ path: 'nomi/functions/bad.ts', content: `const n: number = 'text';\nexport default () => new Response(String(n));\n` }], env: ENV },
  });
  const body = await broken.json();
  assertEquals(body.ok, false);
  assertStringIncludes(body.error, 'nomi/functions/bad.ts:1');
  const still = await call(rt, 'GET', `/run/${PROJECT}/dev/1/hello`, { headers: { 'x-relay-timeout': '5' } });
  assertEquals(still.status, 200);
  await still.body?.cancel();
  await rt.close();
});

Deno.test('a file whose default export is not a function fails the deploy', async () => {
  const rt = await runtime();
  const answer = await call(rt, 'PUT', `/builds/${PROJECT}/dev/1`, { body: { files: [{ path: 'nomi/functions/odd.ts', content: `export default 42;\n` }], env: ENV } });
  const body = await answer.json();
  assertEquals(body.ok, false);
  assertStringIncludes(body.error, 'odd');
  await rt.close();
});

Deno.test('a file that never finishes loading fails the deploy instead of hanging', async () => {
  const rt = await runtime({ initTimeoutMs: 1500 });
  const answer = await call(rt, 'PUT', `/builds/${PROJECT}/dev/1`, {
    body: { files: [{ path: 'nomi/functions/spin.ts', content: `while (true) {}\nexport default () => new Response('never');\n` }], env: ENV },
  });
  const body = await answer.json();
  assertEquals(body.ok, false);
  assertStringIncludes(body.error, 'load');
  await rt.close();
});

Deno.test('a call over its timeout answers 504 and the next call still works', async () => {
  const rt = await runtime();
  const slow = `export default async (req: Request) => {\n  if (new URL(req.url).searchParams.has('slow')) await new Promise(() => {});\n  return new Response('quick');\n};\n`;
  await call(rt, 'PUT', `/builds/${PROJECT}/dev/1`, { body: { files: [{ path: 'nomi/functions/wait.ts', content: slow }], env: ENV } }).then((r) => r.body?.cancel());

  const timedOut = await call(rt, 'GET', `/run/${PROJECT}/dev/1/wait?slow`, { headers: { 'x-relay-timeout': '1' } });
  assertEquals(timedOut.status, 504);
  await timedOut.body?.cancel();
  const quick = await call(rt, 'GET', `/run/${PROJECT}/dev/1/wait`, { headers: { 'x-relay-timeout': '5' } });
  assertEquals(await quick.text(), 'quick');
  await rt.close();
});

Deno.test('a function has no access to the process environment or other files', async () => {
  const rt = await runtime();
  const nosy = `export default async () => {\n  const tried: Record<string, string> = {};\n  try { Deno.env.get('HOME'); tried.env = 'read'; } catch { tried.env = 'blocked'; }\n  try { await Deno.readTextFile('/etc/hostname'); tried.file = 'read'; } catch { tried.file = 'blocked'; }\n  try { new Deno.Command('ls').outputSync(); tried.run = 'ran'; } catch { tried.run = 'blocked'; }\n  return Response.json(tried);\n};\n`;
  await call(rt, 'PUT', `/builds/${PROJECT}/dev/1`, { body: { files: [{ path: 'nomi/functions/nosy.ts', content: nosy }], env: ENV } }).then((r) => r.body?.cancel());
  const answer = await call(rt, 'GET', `/run/${PROJECT}/dev/1/nosy`, { headers: { 'x-relay-timeout': '5' } });
  assertEquals(await answer.json(), { env: 'blocked', file: 'blocked', run: 'blocked' });
  await rt.close();
});

Deno.test('paths outside the functions folder are refused', async () => {
  const rt = await runtime();
  for (const path of ['nomi/functions/../x.ts', '/etc/x.ts', 'nomi/functions/a//b.ts', 'nomi/functions/a\\b.ts', 'nomi/functions/run.sh']) {
    const answer = await call(rt, 'PUT', `/builds/${PROJECT}/dev/1`, { body: { files: [{ path, content: 'export default () => new Response()' }], env: ENV } });
    assertEquals(answer.status, 400, path);
    await answer.body?.cancel();
  }
  const project = await call(rt, 'PUT', `/builds/not-a-project/dev/1`, { body: { files: [], env: ENV } });
  assertEquals(project.status, 400);
  await project.body?.cancel();
  await rt.close();
});

Deno.test('imports of nomi-relay point at the pinned package', () => {
  const sdk = 'npm:nomi-relay@0.1.2';
  assertEquals(rewriteImports(`import { defineFunction } from 'nomi-relay/functions';`, sdk), `import { defineFunction } from 'npm:nomi-relay@0.1.2/functions';`);
  assertEquals(rewriteImports(`import { createClient } from "nomi-relay";`, sdk), `import { createClient } from "npm:nomi-relay@0.1.2";`);
  assertEquals(rewriteImports(`const m = await import('nomi-relay');`, sdk), `const m = await import('npm:nomi-relay@0.1.2');`);
  assertEquals(rewriteImports(`import x from 'nomi-relay-extra';`, sdk), `import x from 'nomi-relay-extra';`);
  assert(cleanPath('nomi/functions/_shared/a.ts') === '_shared/a.ts');
  assert(cleanPath('hello.ts') === 'hello.ts');
});
