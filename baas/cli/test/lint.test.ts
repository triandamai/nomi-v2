import { describe, expect, it } from 'vitest';
import { lintMigrations } from '../src/lint.js';
import { splitStatements } from '../src/sql.js';

const lint = (sql: string, more: string[] = []) =>
  lintMigrations([{ name: '0001_init.sql', sql }, ...more.map((extra, i) => ({ name: `000${i + 2}_x.sql`, sql: extra }))]);

describe('splitStatements', () => {
  it('ignores semicolons in strings, dollar bodies and comments, and tracks lines', () => {
    const statements = splitStatements(`-- first; not a split
create table a (note text default 'x;y');

create function f() returns void as $fn$ begin perform 1; end $fn$ language plpgsql;
/* block; */ select 1;`);
    expect(statements.map((statement) => statement.line)).toEqual([2, 4, 5]);
    expect(statements[0]!.comments).toContain('first; not a split');
    expect(statements[1]!.code).toContain('$body$');
  });
});

describe('lintMigrations', () => {
  it('passes the example from the design doc', () => {
    expect(
      lint(`create table task (
  id bigint generated always as identity primary key,
  owner uuid not null default nomi.uid() references nomi.users(id) on delete cascade,
  title text not null
);
alter table task enable row level security;
create policy "own tasks" on task using (owner = nomi.uid()) with check (owner = nomi.uid());`)
    ).toEqual([]);
  });

  it('flags a table without row-level security', () => {
    const issues = lint('create table note (id int);');
    expect(issues).toEqual([expect.objectContaining({ rule: 'row-level-security', line: 1 })]);
  });

  it('accepts row-level security enabled in a later migration, or a public marker', () => {
    expect(lint('create table note (id int);', ['alter table only "note" enable row level security;'])).toEqual([]);
    expect(lint('-- nomi: public\ncreate table country (code text);')).toEqual([]);
  });

  it('requires a marker on drops that lose data', () => {
    expect(lint('drop table note;').map((issue) => issue.rule)).toEqual(['destructive']);
    expect(lint('alter table task drop column title;').map((issue) => issue.rule)).toEqual(['destructive']);
    expect(lint('-- nomi: destructive\ndrop table note;')).toEqual([]);
    expect(lint('alter table task drop constraint task_title_key;')).toEqual([]);
    expect(lint('alter table task alter column title drop not null;')).toEqual([]);
  });

  it('rejects statements that reach outside the project', () => {
    const rules = (sql: string) => lint(sql).map((issue) => issue.rule);
    expect(rules('create table nomi.evil (id int);')).toContain('outside-project');
    expect(rules('create policy p on nomi.users using (true);')).toEqual(['outside-project']);
    expect(rules('create schema other;')).toEqual(['outside-project']);
    expect(rules('set role postgres;')).toEqual(['outside-project']);
    expect(rules('create function public.f() returns int as $$ select 1 $$ language sql;')).toEqual([]);
  });
});
