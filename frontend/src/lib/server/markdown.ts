import { Marked } from 'marked';
import sanitizeHtml from 'sanitize-html';
import { getSingletonHighlighter, type BundledLanguage, type Highlighter } from 'shiki';

// Dual light/dark themes: shiki emits CSS custom properties (--shiki-light / --shiki-dark) per
// token instead of baking in one theme's literal colors, so highlighted code follows the same
// prefers-color-scheme switch the rest of the app already uses — no client JS involved.
const SHIKI_THEMES = { light: 'github-light', dark: 'github-dark' } as const;

// A curated set covering what's realistically useful in a chat product — not the full bundle,
// which would pull in hundreds of grammars this app will never render.
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
	if (!highlighterPromise) {
		highlighterPromise = getSingletonHighlighter({
			themes: [SHIKI_THEMES.light, SHIKI_THEMES.dark],
			langs: SHIKI_LANGS,
		});
	}
	return highlighterPromise;
}

function escapeHtml(text: string): string {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// Shiki's own output always starts with `<pre class="shiki` — inserting a data-lang attribute
// right after `<pre ` lets the client (MessageBubble.svelte) read the language back out of the
// rendered DOM to build the code block's header, without needing a second data channel.
function withLangAttribute(html: string, lang: string): string {
	return html.replace('<pre ', `<pre data-lang="${escapeHtml(lang)}" `);
}

// Allowlist covers everything marked's GFM output plus shiki's <pre>/<span style="..."> token
// markup can produce. `style` is only permitted on span/pre/code — shiki is the only source of
// that attribute, never a user's raw markdown, which sanitize-html strips regardless of tag.
const SANITIZE_OPTIONS: sanitizeHtml.IOptions = {
	allowedTags: [
		'p',
		'br',
		'hr',
		'strong',
		'em',
		'del',
		'code',
		'pre',
		'span',
		'ul',
		'ol',
		'li',
		'blockquote',
		'a',
		'h1',
		'h2',
		'h3',
		'h4',
		'h5',
		'h6',
		'table',
		'thead',
		'tbody',
		'tr',
		'th',
		'td',
	],
	allowedAttributes: {
		a: ['href', 'title'],
		span: ['style', 'class'],
		pre: ['style', 'class', 'data-lang'],
		code: ['class'],
	},
	allowedSchemes: ['http', 'https', 'mailto'],
};

/** Converts a chat message's markdown source into sanitized HTML, with fenced code blocks
 * syntax-highlighted via shiki. Renders server-side (called from +page.server.ts's load) so no
 * highlighter, grammar, or theme ever ships to the client. Never throws — a markdown/highlighter
 * failure degrades to plain escaped text rather than breaking the whole page load. */
export async function renderMarkdown(source: string): Promise<string> {
	try {
		const hl = await getHighlighter();
		const loadedLangs = new Set(hl.getLoadedLanguages());

		const marked = new Marked({
			gfm: true,
			breaks: true,
		});
		marked.use({
			renderer: {
				code({ text, lang }) {
					const normalizedLang = lang?.trim().split(/\s+/)[0];
					if (normalizedLang && loadedLangs.has(normalizedLang)) {
						try {
							return withLangAttribute(hl.codeToHtml(text, { lang: normalizedLang, themes: SHIKI_THEMES }), normalizedLang);
						} catch {
							// Fall through to the plain-text path below — a grammar edge case shouldn't
							// break rendering the message.
						}
					}
					const displayLang = normalizedLang || 'text';
					return withLangAttribute(
						`<pre class="shiki-fallback"><code>${escapeHtml(text)}</code></pre>`,
						displayLang,
					);
				},
			},
		});

		const html = marked.parse(source, { async: false });
		return sanitizeHtml(html, SANITIZE_OPTIONS);
	} catch {
		return `<p>${escapeHtml(source)}</p>`;
	}
}
