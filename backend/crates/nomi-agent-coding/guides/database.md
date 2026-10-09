# Databases with Drizzle

Only add a database when the app keeps data on a server. Drizzle is the only ORM.

- **Postgres** for real, multi-user apps (accounts, shared data, anything that grows).
- **SQLite** for small, single-user or personal tools.

The Nomi preview runs in the browser (WebContainer): there is no database server and native
modules can't load. So the app connects to the in-browser version of its database when the
`NOMI_PREVIEW` environment variable is set, and to the real one otherwise. Same schema, same
queries.

## Schema

`src/lib/server/db/schema.ts`. Postgres:
```ts
import { pgTable, serial, text, integer, boolean, timestamp } from 'drizzle-orm/pg-core';

export const task = pgTable('task', {
	id: serial('id').primaryKey(),
	title: text('title').notNull(),
	done: boolean('done').notNull().default(false),
	createdAt: timestamp('created_at', { withTimezone: true }).notNull().defaultNow()
});
```
SQLite:
```ts
import { sqliteTable, integer, text } from 'drizzle-orm/sqlite-core';

export const task = sqliteTable('task', {
	id: integer('id').primaryKey({ autoIncrement: true }),
	title: text('title').notNull(),
	done: integer('done', { mode: 'boolean' }).notNull().default(false),
	createdAt: integer('created_at', { mode: 'timestamp' }).notNull().$defaultFn(() => new Date())
});
```
Relations: `relations()` from `drizzle-orm`; foreign keys with `.references(() => other.id, { onDelete: 'cascade' })`.
Types: `typeof task.$inferSelect` / `$inferInsert`.

## Migrations

Never `drizzle-kit push` or `migrate` from the command line (they need a database server). Instead:
1. Change the schema.
2. Run `npm run db:generate` (`drizzle-kit generate`): it writes SQL files to `./drizzle` and
   needs no database.
3. The app applies them on start: `migrateDb()` called from the server's startup hook.

`drizzle.config.ts` (Postgres; for SQLite use `dialect: 'sqlite'` and drop `dbCredentials`):
```ts
import { defineConfig } from 'drizzle-kit';

export default defineConfig({
	schema: './src/lib/server/db/schema.ts',
	out: './drizzle',
	dialect: 'postgresql',
	dbCredentials: { url: process.env.DATABASE_URL ?? '' },
	strict: true
});
```

## Postgres connection (`src/lib/server/db/index.ts`)

Packages: dependencies `drizzle-orm@^0.45.4`, `postgres@^3.4.9`, `@electric-sql/pglite@^0.5.8`;
devDependency `drizzle-kit@^0.31.11`.
```ts
import { mkdirSync } from 'node:fs';
import { PGlite } from '@electric-sql/pglite';
import { drizzle as drizzlePglite } from 'drizzle-orm/pglite';
import { migrate as migratePglite } from 'drizzle-orm/pglite/migrator';
import { drizzle as drizzlePostgres } from 'drizzle-orm/postgres-js';
import { migrate as migratePostgres } from 'drizzle-orm/postgres-js/migrator';
import postgres from 'postgres';
import * as schema from './schema.ts';
import { DATABASE_URL, NOMI_PREVIEW } from '$app/env/private';

// The Nomi preview has no Postgres server: PGlite runs Postgres in WebAssembly.
const preview = Boolean(NOMI_PREVIEW) || !DATABASE_URL;

function pglite() {
	mkdirSync('./.data', { recursive: true });
	return new PGlite('./.data/pglite');
}

export const db = preview ? drizzlePglite(pglite(), { schema }) : drizzlePostgres(postgres(DATABASE_URL!), { schema });

export async function migrateDb() {
	const migrationsFolder = './drizzle';
	if (preview) await migratePglite(db as ReturnType<typeof drizzlePglite<typeof schema>>, { migrationsFolder });
	else await migratePostgres(db as ReturnType<typeof drizzlePostgres<typeof schema>>, { migrationsFolder });
}
```

## SQLite connection (`src/lib/server/db/index.ts`)

Packages: dependencies `drizzle-orm@^0.45.4`, `@libsql/client@^0.18.0`,
`@libsql/client-wasm@^0.18.0`; devDependency `drizzle-kit@^0.31.11`. The native client must only
be imported outside the preview, so both are loaded with `await import(...)`:
```ts
import type { LibSQLDatabase } from 'drizzle-orm/libsql';
import { migrate } from 'drizzle-orm/libsql/migrator';
import * as schema from './schema.ts';
import { DATABASE_URL, NOMI_PREVIEW } from '$app/env/private';

// The Nomi preview can't load native modules: there SQLite runs in WebAssembly, in memory.
async function connect(): Promise<LibSQLDatabase<typeof schema>> {
	if (NOMI_PREVIEW) {
		const { createClient } = await import('@libsql/client-wasm');
		const { drizzle } = await import('drizzle-orm/libsql/wasm');
		return drizzle(createClient({ url: ':memory:' }), { schema });
	}
	const { createClient } = await import('@libsql/client');
	const { drizzle } = await import('drizzle-orm/libsql');
	return drizzle(createClient({ url: DATABASE_URL ?? 'file:local.db' }), { schema });
}

export const db = await connect();

export async function migrateDb() {
	await migrate(db, { migrationsFolder: './drizzle' });
}
```
In the preview, SQLite data lasts until the preview restarts; Postgres (PGlite) data lasts for
the browser session. Seed demo data in `migrateDb()` when a table is empty if the app needs it.

Never use `better-sqlite3`, `pg`, `mysql2` or Docker: they can't run in the preview.
