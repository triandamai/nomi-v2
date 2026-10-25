import { access, mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { lintMigrations } from './lint.js';
import { nextMigrationFile, readMigrations, slug } from './migrations.js';

const EXIT_OK = 0;
export const EXIT_PROBLEMS = 1;
const EXIT_USAGE = 2;
/** Shown checksums are cut to this many characters; the full one is what Relay compares. */
const SHORT_CHECKSUM_LENGTH = 12;

export interface Io {
  cwd: string;
  out: (line: string) => void;
  err: (line: string) => void;
}

const HELP = `nomi-relay: command line for Nomi Relay

Usage: nomi-relay <command> [options]

Commands
  init                      Create nomi/backend.json, nomi/migrations and nomi/functions
  migrate new <name>        Write the next numbered migration, e.g. 0003_add_orders.sql
  migrate lint              Check migrations for missing row-level security, unmarked drops
                            and statements that reach outside the project
  migrate status            List local migrations with their checksums
  functions new <name>      Write nomi/functions/<name>.ts from a starter

Coming with the hosted Relay
  login, link, migrate apply, types, functions serve, functions deploy, secrets set

Options
  --dir <path>              Project folder (default: the current folder)
  -h, --help                Show this help
  -v, --version             Show the version`;

const COMING_SOON = new Set(['login', 'link', 'types', 'secrets', 'migrate apply', 'functions serve', 'functions deploy']);

const BACKEND_JSON = {
  auth: { methods: ['email_code'], redirect_urls: [] },
  storage: { buckets: [] },
  realtime: [],
  functions: {}
};

const FIRST_MIGRATION = `-- Plain SQL, applied once, in order. Never edit a migration after it's applied: add a new one.
-- Every table needs row-level security (or a "-- nomi: public" marker).
`;

function functionStarter(name: string): string {
  return `import { defineFunction } from 'nomi-relay/functions';

// Called at /functions/${name}, or nomi.functions.invoke('${name}', { body }).
export default defineFunction(async (req, ctx) => {
  if (!ctx.user) return new Response('sign in first', { status: 401 });
  const body = req.method === 'GET' ? null : await req.json();
  return Response.json({ ok: true, user: ctx.user.id, body });
});
`;
}

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function writeNew(path: string, content: string, io: Io): Promise<boolean> {
  if (await exists(path)) {
    io.err(`${relative(io.cwd, path)} already exists; leaving it as it is`);
    return false;
  }
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, content);
  io.out(`created ${relative(io.cwd, path)}`);
  return true;
}

async function version(): Promise<string> {
  const pkg = JSON.parse(await readFile(join(dirname(fileURLToPath(import.meta.url)), '..', 'package.json'), 'utf8'));
  return String(pkg.version);
}

export async function run(argv: string[], io: Io): Promise<number> {
  let parsed;
  try {
    parsed = parseArgs({
      args: argv,
      allowPositionals: true,
      options: {
        dir: { type: 'string' },
        help: { type: 'boolean', short: 'h' },
        version: { type: 'boolean', short: 'v' }
      }
    });
  } catch (error) {
    io.err((error as Error).message);
    return EXIT_USAGE;
  }
  const { values, positionals } = parsed;
  if (values.version) {
    io.out(await version());
    return EXIT_OK;
  }
  const [command, sub, ...rest] = positionals;
  if (values.help || !command || command === 'help') {
    io.out(HELP);
    return command || values.help ? EXIT_OK : EXIT_USAGE;
  }

  const root = resolve(io.cwd, values.dir ?? '.');
  const nomiDir = join(root, 'nomi');
  const migrationsDir = join(nomiDir, 'migrations');
  const functionsDir = join(nomiDir, 'functions');

  if (COMING_SOON.has(command) || COMING_SOON.has(`${command} ${sub}`)) {
    io.err(`"${[command, sub].filter(Boolean).join(' ')}" needs the hosted Nomi Relay and isn't available in this release yet.`);
    return EXIT_USAGE;
  }

  switch (`${command} ${sub ?? ''}`.trim()) {
    case 'init': {
      await writeNew(join(nomiDir, 'backend.json'), JSON.stringify(BACKEND_JSON, null, 2) + '\n', io);
      const { migrations } = await readMigrations(migrationsDir);
      if (migrations.length === 0) await writeNew(join(migrationsDir, '0001_init.sql'), FIRST_MIGRATION, io);
      await mkdir(functionsDir, { recursive: true });
      return EXIT_OK;
    }

    case 'migrate new': {
      const name = rest.join(' ');
      if (!slug(name)) {
        io.err('Usage: nomi-relay migrate new <name>');
        return EXIT_USAGE;
      }
      const { migrations } = await readMigrations(migrationsDir);
      await writeNew(join(migrationsDir, nextMigrationFile(migrations, name)), '', io);
      return EXIT_OK;
    }

    case 'migrate lint': {
      const { migrations, problems } = await readMigrations(migrationsDir);
      for (const problem of problems) io.err(`${problem.file}: ${problem.message}`);
      const issues = lintMigrations(migrations.map((migration) => ({ name: migration.file, sql: migration.sql })));
      for (const issue of issues) io.err(`${issue.file}:${issue.line}  ${issue.rule}  ${issue.message}`);
      const count = problems.length + issues.length;
      io.out(count === 0 ? `${migrations.length} migration(s) look good` : `${count} problem(s) found`);
      return count === 0 ? EXIT_OK : EXIT_PROBLEMS;
    }

    case 'migrate status': {
      const { migrations, problems } = await readMigrations(migrationsDir);
      if (migrations.length === 0) io.out('no migrations yet: run "nomi-relay migrate new <name>"');
      for (const migration of migrations) io.out(`${migration.file}  ${migration.checksum.slice(0, SHORT_CHECKSUM_LENGTH)}`);
      for (const problem of problems) io.err(`${problem.file}: ${problem.message}`);
      return problems.length === 0 ? EXIT_OK : EXIT_PROBLEMS;
    }

    case 'functions new': {
      const name = slug(rest.join(' ')).replace(/_/g, '-');
      if (!name) {
        io.err('Usage: nomi-relay functions new <name>');
        return EXIT_USAGE;
      }
      const created = await writeNew(join(functionsDir, `${name}.ts`), functionStarter(name), io);
      return created ? EXIT_OK : EXIT_PROBLEMS;
    }

    default:
      io.err(`Unknown command "${positionals.join(' ')}". Run "nomi-relay --help".`);
      return EXIT_USAGE;
  }
}
