// @vitest-environment jsdom
import { afterEach, describe, expect, it } from 'vitest';
import { flushSync } from 'svelte';
import { citedSources, enrichReply } from '$lib/richReply';

// A reply's text gets the design system's pieces: code blocks, citations, links and @mentions.

afterEach(() => {
	document.body.innerHTML = '';
});

function reply(html: string): HTMLElement {
	const root = document.body.appendChild(document.createElement('div'));
	root.className = 'message-markdown';
	root.innerHTML = html;
	return root;
}

describe('enrichReply', () => {
	it('swaps fenced code for the numbered code block, keeping the code as written', () => {
		const root = reply('<pre data-lang="ts" class="shiki"><code><span>const a = 1</span>\n<span>a + 1</span></code></pre>');
		const undo = enrichReply(root, [], 's');
		flushSync();
		expect(root.querySelector('pre[data-lang]')).toBeNull();
		expect(root.querySelector('figure figcaption')?.textContent).toContain('ts');
		expect(root.querySelectorAll('[data-line-number]').length).toBe(2);
		undo();
	});

	it('turns numbered links into citations pointing at the sources list, other links into hyperlinks', () => {
		const markdown = 'Rent goes up [1](https://kosku.id/a). See [the rules](https://kosku.id/rules).';
		const cited = citedSources(markdown, [{ title: 'Kos Mawar', url: 'https://kosku.id/a' }]);
		const root = reply('<p>Rent goes up <a href="https://kosku.id/a">1</a>. See <a href="https://kosku.id/rules">the rules</a>.</p>');
		const undo = enrichReply(root, cited, 'sources-m1');
		flushSync();
		const links = [...root.querySelectorAll('a')];
		expect(links.map((a) => a.getAttribute('href'))).toEqual(['#sources-m1-1', 'https://kosku.id/rules']);
		expect(links[0].textContent).toContain('1');
		expect(links[0].getAttribute('aria-label') ?? links[0].textContent).toBeTruthy();
		expect(links[1].textContent).toBe('the rules');
		undo();
	});

	it('tints @mentions of the crew in prose, but not in code', () => {
		const root = reply('<p>Ask @Dana to log it, and <code>@Kala</code> stays code.</p>');
		const undo = enrichReply(root, [], 's');
		flushSync();
		expect(root.textContent).toBe('Ask @Dana to log it, and @Kala stays code.');
		expect(root.querySelectorAll('.rich-inline').length).toBe(1);
		expect(root.querySelector('code')?.textContent).toBe('@Kala');
		undo();
	});

	it('shows a file named in inline code as a file mention, and leaves other code alone', () => {
		const root = reply('<p>Logged in <code>rent-2026.xlsx</code>, see <code>Math.round</code>.</p>');
		const undo = enrichReply(root, [], 's');
		flushSync();
		expect(root.querySelectorAll('.rich-inline').length).toBe(1);
		expect(root.textContent).toContain('rent-2026.xlsx');
		expect(root.querySelector('code')?.textContent).toBe('Math.round');
		undo();
	});
});
