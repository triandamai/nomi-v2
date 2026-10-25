/** One SQL statement, with the comments just above it (where `-- nomi:` markers live). */
export interface Statement {
  /** The statement's code, comments removed and whitespace collapsed, lowercased. */
  code: string;
  /** The comments directly before the statement and inside it. */
  comments: string;
  /** 1-based line the statement starts on. */
  line: number;
}

/**
 * Splits SQL into statements on `;`, skipping semicolons inside strings, quoted identifiers,
 * dollar-quoted bodies (`$$ … $$`, `$fn$ … $fn$`) and comments.
 */
export function splitStatements(sql: string): Statement[] {
  const statements: Statement[] = [];
  let code = '';
  let comments = '';
  let line = 1;
  let startLine = 0;
  let i = 0;

  const flush = () => {
    const normalized = code.replace(/\s+/g, ' ').trim().toLowerCase();
    if (normalized) statements.push({ code: normalized, comments, line: startLine || line });
    code = '';
    comments = '';
    startLine = 0;
  };
  const markStart = () => {
    if (!startLine) startLine = line;
  };

  while (i < sql.length) {
    const char = sql[i]!;
    const next = sql[i + 1];

    if (char === '\n') line += 1;

    if (char === '-' && next === '-') {
      const end = sql.indexOf('\n', i);
      const stop = end === -1 ? sql.length : end;
      comments += sql.slice(i, stop) + '\n';
      i = stop;
      continue;
    }
    if (char === '/' && next === '*') {
      const end = sql.indexOf('*/', i + 2);
      const stop = end === -1 ? sql.length : end + 2;
      const block = sql.slice(i, stop);
      comments += block + '\n';
      line += block.split('\n').length - 1;
      i = stop;
      code += ' ';
      continue;
    }
    if (char === "'" || char === '"') {
      markStart();
      let j = i + 1;
      while (j < sql.length) {
        if (sql[j] === char && sql[j + 1] === char) j += 2;
        else if (sql[j] === char) break;
        else j += 1;
      }
      const quoted = sql.slice(i, j + 1);
      line += quoted.split('\n').length - 1;
      // Keep quoted identifiers (they matter for names); blank out string contents.
      code += char === '"' ? quoted : "''";
      i = j + 1;
      continue;
    }
    if (char === '$') {
      const tag = /^\$[A-Za-z_]*\$/.exec(sql.slice(i))?.[0];
      if (tag) {
        markStart();
        const end = sql.indexOf(tag, i + tag.length);
        const stop = end === -1 ? sql.length : end + tag.length;
        line += sql.slice(i, stop).split('\n').length - 1;
        code += ' $body$ ';
        i = stop;
        continue;
      }
    }
    if (char === ';') {
      flush();
      i += 1;
      continue;
    }
    if (!/\s/.test(char)) markStart();
    code += char;
    i += 1;
  }
  flush();
  return statements;
}

export function unquote(identifier: string): string {
  return identifier.replace(/^"|"$/g, '').toLowerCase();
}
