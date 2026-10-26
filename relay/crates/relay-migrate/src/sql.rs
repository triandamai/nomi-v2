//! Splitting a migration into statements, the same way as the CLI (`relay/cli/src/sql.ts`).

/// One SQL statement, with the comments just above and inside it (where `-- nomi:` markers live).
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The statement's code: comments removed, string contents blanked, whitespace collapsed,
    /// lowercased.
    pub code: String,
    pub comments: String,
    /// 1-based line the statement starts on.
    pub line: usize,
}

/// Splits on `;`, skipping semicolons inside strings, quoted identifiers, dollar-quoted bodies
/// and comments.
pub fn split_statements(sql: &str) -> Vec<Statement> {
    let chars: Vec<char> = sql.chars().collect();
    let mut statements = Vec::new();
    let mut code = String::new();
    let mut comments = String::new();
    let mut line = 1usize;
    let mut start_line = 0usize;
    let mut i = 0usize;

    let mut flush = |code: &mut String, comments: &mut String, start_line: &mut usize, line: usize| {
        let normalized = code.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
        if !normalized.is_empty() {
            statements.push(Statement { code: normalized, comments: comments.clone(), line: if *start_line > 0 { *start_line } else { line } });
        }
        code.clear();
        comments.clear();
        *start_line = 0;
    };
    let count_lines = |text: &[char]| text.iter().filter(|c| **c == '\n').count();

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '\n' {
            line += 1;
        }
        if c == '-' && next == Some('-') {
            let end = chars[i..].iter().position(|c| *c == '\n').map(|p| i + p).unwrap_or(chars.len());
            comments.extend(&chars[i..end]);
            comments.push('\n');
            i = end;
            continue;
        }
        if c == '/' && next == Some('*') {
            let end = (i + 2..chars.len().saturating_sub(1)).find(|&j| chars[j] == '*' && chars[j + 1] == '/').map(|j| j + 2).unwrap_or(chars.len());
            comments.extend(&chars[i..end]);
            comments.push('\n');
            line += count_lines(&chars[i..end]);
            code.push(' ');
            i = end;
            continue;
        }
        if c == '\'' || c == '"' {
            if start_line == 0 {
                start_line = line;
            }
            let mut j = i + 1;
            while j < chars.len() {
                if chars[j] == c && chars.get(j + 1) == Some(&c) {
                    j += 2;
                } else if chars[j] == c {
                    break;
                } else {
                    j += 1;
                }
            }
            let end = (j + 1).min(chars.len());
            line += count_lines(&chars[i..end]);
            // Quoted identifiers matter for names; string contents don't.
            if c == '"' {
                code.extend(&chars[i..end]);
            } else {
                code.push_str("''");
            }
            i = end;
            continue;
        }
        if c == '$' {
            let tag_len = chars[i + 1..].iter().take_while(|c| c.is_ascii_alphabetic() || **c == '_').count();
            if chars.get(i + 1 + tag_len) == Some(&'$') {
                if start_line == 0 {
                    start_line = line;
                }
                let tag: Vec<char> = chars[i..i + tag_len + 2].to_vec();
                let after = i + tag.len();
                let end = (after..=chars.len().saturating_sub(tag.len()))
                    .find(|&j| chars[j..j + tag.len()] == tag[..])
                    .map(|j| j + tag.len())
                    .unwrap_or(chars.len());
                line += count_lines(&chars[i..end]);
                code.push_str(" $body$ ");
                i = end;
                continue;
            }
        }
        if c == ';' {
            flush(&mut code, &mut comments, &mut start_line, line);
            i += 1;
            continue;
        }
        if !c.is_whitespace() && start_line == 0 {
            start_line = line;
        }
        code.push(c);
        i += 1;
    }
    flush(&mut code, &mut comments, &mut start_line, line);
    statements
}

/// `"Note"` → `note`.
pub fn unquote(identifier: &str) -> String {
    identifier.trim_matches('"').to_lowercase()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_semicolons_in_strings_dollar_bodies_and_comments_and_tracks_lines() {
        let statements = split_statements(
            "-- first; not a split\ncreate table a (note text default 'x;y');\n\ncreate function f() returns void as $fn$ begin perform 1; end $fn$ language plpgsql;\n/* block; */ select 1;",
        );
        assert_eq!(statements.iter().map(|s| s.line).collect::<Vec<_>>(), vec![2, 4, 5]);
        assert!(statements[0].comments.contains("first; not a split"));
        assert!(statements[1].code.contains("$body$"));
    }
}
