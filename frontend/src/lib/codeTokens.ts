// Colours code for CodeBlock (Chat responses → Code), whose syntax colours are the nm-code-* tokens.
// A light, language-agnostic pass: comments, strings, numbers, keywords, types and calls. It never
// changes the text, so copying a highlighted block gives back exactly the code.
import type { CodeKind, CodeLine, CodeToken } from '$lib/components/ds/chat/code';

const KEYWORDS = new Set(
	(
		'abstract as async await break case catch class const continue def default defer del delete do elif else enum export extends ' +
		'false finally fn for from func function go if impl import in instanceof interface is lambda let loop match mod module mut new nil ' +
		'none not null or and package pass priv pub raise return select self static struct super switch this throw true try type typeof ' +
		'undefined union unsafe use var void where while with yield None True False SELECT FROM WHERE INSERT INTO UPDATE DELETE JOIN ' +
		'LEFT RIGHT INNER OUTER ON GROUP BY ORDER LIMIT AND OR NOT NULL AS CREATE TABLE VALUES SET'
	).split(' '),
);

/** Languages whose comments start with `#` rather than `//`. */
const HASH_COMMENTS = new Set(['python', 'py', 'bash', 'sh', 'shell', 'zsh', 'ruby', 'rb', 'yaml', 'yml', 'toml', 'dockerfile', 'r', 'perl', 'makefile']);
const SQL = new Set(['sql', 'postgres', 'postgresql', 'mysql', 'sqlite']);

function lineComment(language: string): RegExp {
	const lang = language.toLowerCase();
	if (HASH_COMMENTS.has(lang)) return /#.*/y;
	if (SQL.has(lang)) return /--.*/y;
	return /\/\/.*/y;
}

const STRING = /"(?:[^"\\]|\\.)*"?|'(?:[^'\\]|\\.)*'?|`(?:[^`\\]|\\.)*`?/y;
const NUMBER = /\b(?:0x[\da-fA-F_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:e[+-]?\d+)?)\b/y;
const WORD = /[A-Za-z_$][\w$]*/y;
const PUNCT = /[{}()[\];,.:<>=+\-*/%!&|^~?@]+/y;

function at(re: RegExp, text: string, index: number): string | null {
	re.lastIndex = index;
	const match = re.exec(text);
	return match && match.index === index && match[0] ? match[0] : null;
}

/** The code as CodeBlock's lines of coloured tokens. */
export function highlight(code: string, language = ''): CodeLine[] {
	const comment = lineComment(language);
	let inBlock = false; // inside /* … */, which can span lines
	return code.split('\n').map((text) => {
		const tokens: CodeToken[] = [];
		const push = (value: string, kind?: CodeKind) => {
			const last = tokens.at(-1);
			if (last && last.kind === kind) last.text += value;
			else tokens.push(kind ? { text: value, kind } : { text: value });
		};
		let i = 0;
		while (i < text.length) {
			const opens = !inBlock && text.startsWith('/*', i);
			if (inBlock || opens) {
				const end = text.indexOf('*/', opens ? i + 2 : i);
				const stop = end === -1 ? text.length : end + 2;
				push(text.slice(i, stop), 'comment');
				inBlock = end === -1;
				i = stop;
				continue;
			}
			let match: string | null;
			if ((match = at(comment, text, i))) {
				push(match, 'comment');
			} else if ((match = at(STRING, text, i))) {
				push(match, 'string');
			} else if ((match = at(NUMBER, text, i))) {
				push(match, 'number');
			} else if ((match = at(WORD, text, i))) {
				const next = text.slice(i + match.length).trimStart();
				const kind: CodeKind | undefined = KEYWORDS.has(match) ? 'keyword' : next.startsWith('(') ? 'fn' : /^[A-Z][a-z]/.test(match) ? 'type' : undefined;
				push(match, kind);
			} else if ((match = at(PUNCT, text, i))) {
				push(match);
			} else {
				match = text[i];
				push(match);
			}
			i += match.length;
		}
		return tokens;
	});
}
