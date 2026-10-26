import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { run, type Spawn } from '../src/run.js';
import { serveEntry } from '../src/serve.js';

let cwd: string;
let out: string[];
let err: string[];
let spawned: { command: string; args: string[]; env: Record<string, string | undefined>; entry: string }[];
const spawn: Spawn = async (command, args, env) => {
  const entry = await readFile(args[args.length - 1]!, 'utf8');
  spawned.push({ command, args, env, entry });
  return 0;
};
const ENV = { NOMI_URL: 'https://shop-dev.relay.test', NOMI_ANON_KEY: 'anon', NOMI_SERVICE_KEY: 'service', STRIPE_SECRET: 'sk' };
const cli = (env: Record<string, string>, ...argv: string[]) => run(argv, { cwd, out: (l) => out.push(l), err: (l) => err.push(l), env, spawn });

beforeEach(async () => {
  cwd = await mkdtemp(join(tmpdir(), 'nomi-relay-serve-'));
  out = [];
  err = [];
  spawned = [];
  await mkdir(join(cwd, 'nomi/functions/_shared'), { recursive: true });
  await writeFile(join(cwd, 'nomi/functions/place-order.ts'), "export default () => new Response('ok');\n");
  await writeFile(join(cwd, 'nomi/functions/_shared/money.ts'), 'export const cents = 1;\n');
});
afterEach(() => rm(cwd, { recursive: true, force: true }));

describe('functions serve', () => {
  it("runs the project's functions on Deno with only the network, the environment and its own folder", async () => {
    expect(await cli(ENV, 'functions', 'serve', '--port', '9000')).toBe(0);

    expect(spawned).toHaveLength(1);
    const { command, args, env, entry } = spawned[0]!;
    expect(command).toBe('deno');
    expect(args.slice(0, 4)).toEqual(['run', '--allow-net', '--allow-env', `--allow-read=${join(cwd, 'nomi/functions')}`]);
    expect(env.PORT).toBe('9000');
    expect(env.STRIPE_SECRET).toBe('sk');
    expect(entry).toContain(`'place-order': () => import(${JSON.stringify(`file://${join(cwd, 'nomi/functions/place-order.ts')}`)})`);
    expect(entry).not.toContain('money');
    expect(out.join('\n')).toContain('http://localhost:9000/functions/place-order');
  });

  it("says what's missing: the backend's address and keys, or any function", async () => {
    expect(await cli({ NOMI_URL: 'x' }, 'functions', 'serve')).toBe(2);
    expect(err.join('\n')).toContain('NOMI_ANON_KEY');
    await rm(join(cwd, 'nomi/functions/place-order.ts'));
    err = [];
    expect(await cli(ENV, 'functions', 'serve')).toBe(1);
    expect(err.join('\n')).toContain('no functions');
    expect(spawned).toHaveLength(0);
  });

  it('the entry routes /functions/<name> and passes the token’s claims on', () => {
    const entry = serveEntry({ 'place-order': '/p/nomi/functions/place-order.ts' }, 'npm:nomi-relay@0.1.2');
    expect(entry).toContain("from 'npm:nomi-relay@0.1.2/functions'");
    expect(entry).toContain("'x-nomi-claims'");
    expect(entry).toContain('/functions/');
  });
});
