import { createHash } from 'node:crypto';
import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

/** Migrations are `0001_init.sql`, `0002_orders.sql`…: a zero-padded number, then a name. */
export const MIGRATION_NAME = /^(\d{4,})_([a-z0-9_]+)\.sql$/;
const NUMBER_WIDTH = 4;

export interface Migration {
  number: number;
  name: string;
  file: string;
  sql: string;
  checksum: string;
}

export interface MigrationSet {
  migrations: Migration[];
  /** Files in the folder that aren't valid migrations, with why. */
  problems: { file: string; message: string }[];
}

export function checksum(sql: string): string {
  return createHash('sha256').update(sql).digest('hex');
}

export async function readMigrations(dir: string): Promise<MigrationSet> {
  let entries: string[];
  try {
    entries = await readdir(dir);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return { migrations: [], problems: [] };
    throw error;
  }

  const migrations: Migration[] = [];
  const problems: MigrationSet['problems'] = [];
  for (const file of entries.sort()) {
    if (file.startsWith('.')) continue;
    const match = MIGRATION_NAME.exec(file);
    if (!match) {
      problems.push({ file, message: 'not a migration name: use 0001_name.sql (lowercase letters, digits, underscores)' });
      continue;
    }
    const sql = await readFile(join(dir, file), 'utf8');
    migrations.push({ number: Number(match[1]), name: match[2]!, file, sql, checksum: checksum(sql) });
  }
  migrations.sort((a, b) => a.number - b.number);

  for (let i = 1; i < migrations.length; i++) {
    if (migrations[i]!.number === migrations[i - 1]!.number) {
      problems.push({ file: migrations[i]!.file, message: `has the same number as ${migrations[i - 1]!.file}` });
    }
  }
  return { migrations, problems };
}

/** `Add Orders!` → `add_orders`. */
export function slug(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
}

export function nextMigrationFile(existing: Migration[], name: string): string {
  const number = (existing.at(-1)?.number ?? 0) + 1;
  return `${String(number).padStart(NUMBER_WIDTH, '0')}_${slug(name)}.sql`;
}
