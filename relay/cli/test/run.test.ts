import { mkdtemp, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { run } from '../src/run.js';

let cwd: string;
let out: string[];
let err: string[];
const cli = (...argv: string[]) => run(argv, { cwd, out: (line) => out.push(line), err: (line) => err.push(line) });

beforeEach(async () => {
  cwd = await mkdtemp(join(tmpdir(), 'nomi-relay-cli-'));
  out = [];
  err = [];
});
afterEach(() => rm(cwd, { recursive: true, force: true }));

describe('nomi-relay', () => {
  it('prints its version', async () => {
    expect(await cli('--version')).toBe(0);
    expect(out[0]).toMatch(/^\d+\.\d+\.\d+/);
  });

  it('init lays out the nomi folder and leaves existing files alone', async () => {
    expect(await cli('init')).toBe(0);
    expect(await readdir(join(cwd, 'nomi'))).toEqual(expect.arrayContaining(['backend.json', 'migrations', 'functions']));
    expect(JSON.parse(await readFile(join(cwd, 'nomi/backend.json'), 'utf8')).auth.methods).toEqual(['email_code']);

    await writeFile(join(cwd, 'nomi/backend.json'), '{"keep":true}');
    await cli('init');
    expect(await readFile(join(cwd, 'nomi/backend.json'), 'utf8')).toBe('{"keep":true}');
  });

  it('migrate new numbers after the last migration', async () => {
    await cli('init');
    await cli('migrate', 'new', 'Add Orders!');
    await cli('migrate', 'new', 'tags');
    expect((await readdir(join(cwd, 'nomi/migrations'))).sort()).toEqual(['0001_init.sql', '0002_add_orders.sql', '0003_tags.sql']);
  });

  it('migrate lint fails on problems and passes when they are fixed', async () => {
    await cli('init');
    await writeFile(join(cwd, 'nomi/migrations/0002_note.sql'), 'create table note (id int);\n');
    expect(await cli('migrate', 'lint')).toBe(1);
    expect(err.join('\n')).toContain('0002_note.sql:1  row-level-security');

    await writeFile(join(cwd, 'nomi/migrations/0003_rls.sql'), 'alter table note enable row level security;\n');
    err = [];
    expect(await cli('migrate', 'lint')).toBe(0);
    expect(err).toEqual([]);
  });

  it('migrate lint reports badly named and duplicate-numbered files', async () => {
    await cli('init');
    await writeFile(join(cwd, 'nomi/migrations/0001_again.sql'), '');
    await writeFile(join(cwd, 'nomi/migrations/orders.sql'), '');
    expect(await cli('migrate', 'lint')).toBe(1);
    expect(err.join('\n')).toMatch(/same number/);
    expect(err.join('\n')).toMatch(/orders\.sql: not a migration name/);
  });

  it('functions new writes a starter once', async () => {
    expect(await cli('functions', 'new', 'Place Order')).toBe(0);
    expect(await readFile(join(cwd, 'nomi/functions/place-order.ts'), 'utf8')).toContain("from 'nomi-relay/functions'");
    expect(await cli('functions', 'new', 'place-order')).toBe(1);
  });

  it('says plainly which commands need the hosted Relay', async () => {
    expect(await cli('migrate', 'apply')).toBe(2);
    expect(err[0]).toMatch(/isn't available in this release/);
  });

  it('rejects unknown commands', async () => {
    expect(await cli('frobnicate')).toBe(2);
  });
});
