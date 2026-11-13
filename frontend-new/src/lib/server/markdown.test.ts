import { describe, expect, it } from 'vitest';
import { renderMarkdown } from './markdown';

// Ported from the old app's markdown tests (frontend/src/lib/server/markdown.test.ts).
describe('renderMarkdown', () => {
	it('renders basic markdown formatting', async () => {
		const html = await renderMarkdown('**bold** and *italic* and a [link](https://example.com)');
		expect(html).toContain('<strong>bold</strong>');
		expect(html).toContain('<em>italic</em>');
		expect(html).toContain('<a href="https://example.com" target="_blank" rel="noopener noreferrer">link</a>');
	});

	it('renders lists, tables and quotes', async () => {
		const html = await renderMarkdown('- one\n- two\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n> said');
		expect(html).toContain('<ul>');
		expect(html).toContain('<table>');
		expect(html).toContain('<blockquote>');
	});

	it('syntax-highlights a fenced code block with a known language', async () => {
		const html = await renderMarkdown('```javascript\nconst x = 1;\n```');
		expect(html).toContain('class="shiki');
		expect(html).toContain('const');
		expect(html).toContain('shiki-dark');
	});

	it('falls back to a plain escaped code block for an unknown language', async () => {
		const html = await renderMarkdown('```not-a-real-language\nhello\n```');
		expect(html).toContain('shiki-fallback');
		expect(html).toContain('hello');
		expect(html).not.toMatch(/class="shiki"/);
	});

	it('strips a script tag embedded directly in the message', async () => {
		const html = await renderMarkdown('hi <script>alert(1)</script> there');
		expect(html).not.toContain('<script');
		expect(html).not.toContain('alert(1)');
	});

	it('strips an inline event-handler attribute from a raw img tag', async () => {
		const html = await renderMarkdown('<img src="x" onerror="alert(1)">');
		expect(html).not.toContain('onerror');
	});

	it('strips a javascript: URL from a markdown link', async () => {
		const html = await renderMarkdown('[click me](javascript:alert(1))');
		expect(html).not.toContain('javascript:');
	});

	it('preserves shiki-generated inline styles on highlighted code while still sanitizing', async () => {
		const html = await renderMarkdown('```javascript\nconst x = 1;\n```');
		expect(html).toMatch(/<span style="/);
	});

	it('tags a code block with its language, or "text" without one', async () => {
		expect(await renderMarkdown('```rust\nfn main() {}\n```')).toContain('data-lang="rust"');
		expect(await renderMarkdown('```not-a-real-language\nhello\n```')).toContain('data-lang="not-a-real-language"');
		expect(await renderMarkdown('```\nhello\n```')).toContain('data-lang="text"');
	});

	it('escapes an attempted attribute breakout in an unrecognized language tag', async () => {
		const html = await renderMarkdown('```"><img src=x onerror=alert(1)\nhello\n```');
		expect(html).not.toContain('onerror');
		expect(html).not.toContain('<img');
	});

	it('keeps line breaks the crew wrote', async () => {
		expect(await renderMarkdown('one\ntwo')).toContain('one<br />two');
	});
});
