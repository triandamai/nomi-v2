import { splitStatements, unquote, type Statement } from './sql.js';

export interface MigrationFile {
  name: string;
  sql: string;
}

export type Rule = 'row-level-security' | 'destructive' | 'outside-project';

export interface Issue {
  file: string;
  line: number;
  rule: Rule;
  message: string;
}

/** Marks a table that's meant to be readable without row-level security. */
export const PUBLIC_MARKER = 'nomi: public';
/** Marks a drop that's meant to lose data. */
export const DESTRUCTIVE_MARKER = 'nomi: destructive';

/** The only schema a project's migrations may create things in by name; unqualified names land there too. */
const OWN_SCHEMA = 'public';

const IDENT = String.raw`(?:"[^"]+"|[a-z_][a-z0-9_$]*)`;
const QUALIFIED = String.raw`(?:(${IDENT})\.)?(${IDENT})`;

const CREATE_TABLE = new RegExp(
  String.raw`^create (?:unlogged )?table (?:if not exists )?${QUALIFIED}`
);
const ENABLE_RLS = new RegExp(String.raw`^alter table (?:if exists )?(?:only )?${QUALIFIED} enable row level security`);
const DROP_TABLE = /^drop table /;
const DROP_COLUMN = /^alter table .* drop (?:column )?(?!constraint\b|default\b|not null\b|identity\b|expression\b)/;
const QUALIFIED_DDL = new RegExp(
  String.raw`^(?:create|alter|drop) (?:or replace )?(?:unique )?(?:unlogged )?(?:materialized )?` +
    String.raw`(?:table|view|function|procedure|index|type|trigger|sequence|domain|aggregate) ` +
    String.raw`(?:if (?:not )?exists )?(?:concurrently )?(?:only )?(${IDENT})\.`
);
const QUALIFIED_ON = new RegExp(String.raw`^create (?:policy|trigger|(?:unique )?index) .*? on (?:only )?(${IDENT})\.${IDENT}`);
const FORBIDDEN: { pattern: RegExp; what: string }[] = [
  { pattern: /^(?:create|alter|drop) schema\b/, what: 'creating, changing or dropping a schema' },
  { pattern: /^(?:create|alter|drop) (?:role|user|group)\b/, what: 'managing database roles' },
  { pattern: /^(?:create|alter|drop) database\b/, what: 'managing databases' },
  { pattern: /^(?:set|reset) (?:local )?(?:role|session authorization|search_path)\b/, what: 'switching role or search_path' },
  { pattern: /^alter system\b/, what: 'changing server settings' },
  { pattern: /^(?:create|alter|drop) event trigger\b/, what: 'event triggers' },
  { pattern: /^copy .* (?:from|to) program\b/, what: 'running programs on the server' }
];

function hasMarker(statement: Statement, marker: string): boolean {
  return statement.comments.toLowerCase().includes(marker);
}

/**
 * Checks migrations the way the BaaS does before applying them. This is an early check for the
 * editor and CI; the BaaS runs the authoritative one.
 */
export function lintMigrations(files: MigrationFile[]): Issue[] {
  const issues: Issue[] = [];
  const parsed = files.map((file) => ({ file, statements: splitStatements(file.sql) }));

  const rlsEnabled = new Set<string>();
  for (const { statements } of parsed) {
    for (const statement of statements) {
      const match = ENABLE_RLS.exec(statement.code);
      if (match) rlsEnabled.add(unquote(match[2]!));
    }
  }

  for (const { file, statements } of parsed) {
    for (const statement of statements) {
      const at = { file: file.name, line: statement.line };

      const table = CREATE_TABLE.exec(statement.code);
      if (table) {
        const name = unquote(table[2]!);
        if (!rlsEnabled.has(name) && !hasMarker(statement, PUBLIC_MARKER)) {
          issues.push({
            ...at,
            rule: 'row-level-security',
            message: `table "${name}" has no row-level security: add "alter table ${name} enable row level security;" and policies, or mark it "-- ${PUBLIC_MARKER}"`
          });
        }
      }

      if ((DROP_TABLE.test(statement.code) || DROP_COLUMN.test(statement.code)) && !hasMarker(statement, DESTRUCTIVE_MARKER)) {
        issues.push({
          ...at,
          rule: 'destructive',
          message: `this drops data: mark it "-- ${DESTRUCTIVE_MARKER}" if that's intended`
        });
      }

      const schema = QUALIFIED_DDL.exec(statement.code)?.[1] ?? QUALIFIED_ON.exec(statement.code)?.[1];
      if (schema && unquote(schema) !== OWN_SCHEMA) {
        issues.push({
          ...at,
          rule: 'outside-project',
          message: `migrations may only define things in the project's own schema, not "${unquote(schema)}"`
        });
      }
      for (const { pattern, what } of FORBIDDEN) {
        if (pattern.test(statement.code)) {
          issues.push({ ...at, rule: 'outside-project', message: `migrations can't do ${what}` });
        }
      }
    }
  }
  return issues;
}
