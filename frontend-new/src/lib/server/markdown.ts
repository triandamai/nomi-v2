import { Marked } from 'marked';
import sanitizeHtml from 'sanitize-html';
import { getSingletonHighlighter, type BundledLanguage, type Highlighter } from 'shiki';

// A chat message's markdown as sanitised HTML, rendered on the server (a page load, or one
// message fetched when it lands), so no highlighter, grammar or theme ships to the browser.
// Ported from the old app's markdown (frontend/src/lib/server/markdown.ts). Links open in a new
// tab. Code is highlighted in both themes at once (shiki's --shiki-light / --shiki-dark custom
// properties), which the Prose component picks between by the page's theme.

const SHIKI_THEMES = { light: 'github-light', dark: 'github-dark' } as const;

// The languages a chat realistically shows, not the full bundle.
const SHIKI_LANGS: BundledLanguage[] = [
	'javascript',
	'typescript',
	'jsx',
	'tsx',
	'json',
	'python',
	'rust',
	'go',
	'java',
	'kotlin',
	'bash',
	'shell',
	'sql',
	'html',
	'css',
	'yaml',
	'toml',
	'markdown',
	'diff',
	'dockerfile',
];

let highlighterPromise: Promise<Highlighter> | undefined;

function getHighlighter(): Promise<Highlighter> {
	highlighterPromise ??= getSingletonHighlighter({ themes: [SHIKI_THEMES.light, SHIKI_THEMES.dark], langs: SHIKI_LANGS });
	return highlighterPromise;
}

function escapeHtml(text: string): string {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** Shiki's output starts with `<pre class="shiki`: the language goes on it, for the block's label. */
function withLangAttribute(html: string, lang: string): string {
	return html.replace('<pre ', `<pre data-lang="${escapeHtml(lang)}" `);
}

// Everything marked's GFM output and shiki's token markup can produce. `style` only on span and
// pre: shiki is its only source, and sanitize-html strips it from anything a message itself says.
const SANITIZE_OPTIONS: sanitizeHtml.IOptions = {
	allowedTags: ['p', 'br', 'hr', 'strong', 'em', 'del', 'code', 'pre', 'span', 'ul', 'ol', 'li', 'blockquote', 'a', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'table', 'thead', 'tbody', 'tr', 'th', 'td'],
	allowedAttributes: {
		a: ['href', 'title', 'target', 'rel'],
		span: ['style', 'class'],
		pre: ['style', 'class', 'data-lang'],
		code: ['class'],
		th: ['align'],
		td: ['align'],
	},
	allowedSchemes: ['http', 'https', 'mailto'],
	transformTags: { a: sanitizeHtml.simpleTransform('a', { target: '_blank', rel: 'noopener noreferrer' }) },
};

/** Never throws: a markdown or highlighter failure degrades to the escaped text. */
export async function renderMarkdown(source: string): Promise<string> {
	try {
		const highlighter = await getHighlighter();
		const loaded = new Set(highlighter.getLoadedLanguages());
		const marked = new Marked({ gfm: true, breaks: true });
		marked.use({
			renderer: {
				code({ text, lang }) {
					const language = lang?.trim().split(/\s+/)[0];
					if (language && loaded.has(language)) {
						try {
							return withLangAttribute(highlighter.codeToHtml(text, { lang: language, themes: SHIKI_THEMES }), language);
						} catch {
							// A grammar edge case falls through to the plain block below.
						}
					}
					return withLangAttribute(`<pre class="shiki-fallback"><code>${escapeHtml(text)}</code></pre>`, language || 'text');
				},
			},
		});
		return sanitizeHtml(marked.parse(source, { async: false }), SANITIZE_OPTIONS);
	} catch {
		return `<p>${escapeHtml(source)}</p>`;
	}
}
