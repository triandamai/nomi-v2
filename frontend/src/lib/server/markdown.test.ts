import { describe, expect, it } from 'vitest';
import { renderMarkdown } from './markdown';

describe('renderMarkdown', () => {
	it('renders basic markdown formatting', async () => {
		const html = await renderMarkdown('**bold** and *italic* and a [link](https://example.com)');
		expect(html).toContain('<strong>bold</strong>');
		expect(html).toContain('<em>italic</em>');
		expect(html).toContain('<a href="https://example.com">link</a>');
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
		// Not a real shiki-highlighted block — "shiki-fallback" itself starts with "shiki", so
		// this checks for the exact highlighted-block class, not just the substring "shiki".
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
		// Shiki's dual-theme output relies on a style attribute for its color tokens — sanitize-html
		// must allow that through for span/pre, or highlighting silently breaks.
		expect(html).toMatch(/<span style="/);
	});

	it('tags a highlighted code block with its language for the client-side header to read', async () => {
		const html = await renderMarkdown('```rust\nfn main() {}\n```');
		expect(html).toContain('data-lang="rust"');
	});

	it('tags a fallback code block with its (unrecognized) language too', async () => {
		const html = await renderMarkdown('```not-a-real-language\nhello\n```');
		expect(html).toContain('data-lang="not-a-real-language"');
	});

	it('tags a fenceless/no-language code block with a "text" fallback label', async () => {
		const html = await renderMarkdown('```\nhello\n```');
		expect(html).toContain('data-lang="text"');
	});

	it('escapes an attempted attribute breakout in an unrecognized language tag', async () => {
		const html = await renderMarkdown('```"><img src=x onerror=alert(1)\nhello\n```');
		expect(html).not.toContain('onerror');
		expect(html).not.toContain('<img');
	});
});
