import { getSingletonHighlighter, type BundledLanguage, type Highlighter } from 'shiki';
import { plainLines, type CodeKind, type CodeLine } from '$lib/ds/chat/code';

// The code view's syntax colours, worked out on the server like the chat's code blocks, so no
// highlighter or grammar ships to the browser. Shiki reads the file; what each piece is (its
// TextMate scope) becomes one of the code view's seven kinds, drawn in the theme's nm-code-*
// colours. A file it can't read, or one too big to be worth it, shows as plain lines.

/** Past this many characters a file is shown plain: highlighting it would hold the request up. */
export const MAX_HIGHLIGHT_CHARS = 200_000;

/** Only its scopes are used, not its colours. */
const THEME = 'github-dark';

const BY_EXTENSION: Record<string, BundledLanguage> = {
	ts: 'typescript',
	mts: 'typescript',
	cts: 'typescript',
	tsx: 'tsx',
	js: 'javascript',
	mjs: 'javascript',
	cjs: 'javascript',
	jsx: 'jsx',
	svelte: 'svelte',
	vue: 'vue',
	json: 'json',
	jsonc: 'jsonc',
	html: 'html',
	css: 'css',
	scss: 'scss',
	md: 'markdown',
	py: 'python',
	rs: 'rust',
	go: 'go',
	java: 'java',
	kt: 'kotlin',
	sh: 'bash',
	bash: 'bash',
	sql: 'sql',
	yml: 'yaml',
	yaml: 'yaml',
	toml: 'toml',
	prisma: 'prisma',
};
const BY_NAME: Record<string, BundledLanguage> = { dockerfile: 'dockerfile' };

/** The language a file is written in, from its name; null when the code view doesn't colour it. */
export function languageOf(path: string): BundledLanguage | null {
	const name = path.split('/').at(-1)?.toLowerCase() ?? '';
	if (BY_NAME[name]) return BY_NAME[name];
	const dot = name.lastIndexOf('.');
	return dot > 0 ? (BY_EXTENSION[name.slice(dot + 1)] ?? null) : null;
}

// Most specific first: the innermost scope that says what a piece is wins.
const KINDS: [RegExp, CodeKind][] = [
	[/^comment\b/, 'comment'],
	[/^string\b|^constant\.character\.escape\b/, 'string'],
	[/^constant\.numeric\b/, 'number'],
	[/^entity\.name\.function\b|^support\.function\b|^meta\.function-call\.method\b/, 'fn'],
	[/^entity\.name\.(?:type|class|tag)\b|^support\.(?:type|class)\b|^entity\.other\.inherited-class\b/, 'type'],
	[/^keyword\b|^storage\b|^constant\.language\b|^variable\.language\b/, 'keyword'],
	[/^punctuation\b|^meta\.brace\b/, 'punct'],
];

/** What a piece of code is, from its scopes (outermost first); nothing when it's plain. */
export function kindOf(scopes: string[]): CodeKind | undefined {
	for (const scope of [...scopes].reverse()) {
		const found = KINDS.find(([pattern]) => pattern.test(scope));
		if (found) return found[1];
	}
	return undefined;
}

let highlighterPromise: Promise<Highlighter> | undefined;

async function highlighterFor(language: BundledLanguage): Promise<Highlighter> {
	highlighterPromise ??= getSingletonHighlighter({ themes: [THEME], langs: [] });
	const highlighter = await highlighterPromise;
	if (!highlighter.getLoadedLanguages().includes(language)) await highlighter.loadLanguage(language);
	return highlighter;
}

/** A file's lines in the code view's tokens: coloured where its language is known, plain otherwise. */
export async function highlightLines(code: string, path: string): Promise<CodeLine[]> {
	const language = languageOf(path);
	if (!language || code.length > MAX_HIGHLIGHT_CHARS) return plainLines(code);
	try {
		const highlighter = await highlighterFor(language);
		const { tokens } = highlighter.codeToTokens(code, { lang: language, theme: THEME, includeExplanation: true });
		return tokens.map((line) => {
			const out: CodeLine = [];
			for (const token of line) {
				const parts = token.explanation?.length ? token.explanation.map((part) => ({ text: part.content, scopes: part.scopes.map((scope) => scope.scopeName) })) : [{ text: token.content, scopes: [] }];
				for (const part of parts) {
					const kind = kindOf(part.scopes);
					const last = out.at(-1);
					if (last && last.kind === kind) last.text += part.text;
					else out.push(kind ? { text: part.text, kind } : { text: part.text });
				}
			}
			return out;
		});
	} catch {
		return plainLines(code);
	}
}
