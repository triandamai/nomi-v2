import { describe, expect, it } from 'vitest';
import { relativize } from '../../../scripts/docs/relative-assets.mjs';

describe('the static docs’ own files, from any path', () => {
	it('points a page’s root references at its own folder’s depth', () => {
		const html = '<img src="/app/immutable/assets/receipt.jpg" alt=""><link href="/app/immutable/assets/x.css"><a href="/elsewhere">';
		expect(relativize(html, 1)).toBe('<img src="../app/immutable/assets/receipt.jpg" alt=""><link href="../app/immutable/assets/x.css"><a href="/elsewhere">');
		expect(relativize(html, 0)).toBe('<img src="./app/immutable/assets/receipt.jpg" alt=""><link href="./app/immutable/assets/x.css"><a href="/elsewhere">');
		expect(relativize('<img srcset="/app/immutable/assets/a.jpg 1x, /app/immutable/assets/b.jpg 2x">', 2)).toBe('<img srcset="../../app/immutable/assets/a.jpg 1x, ../../app/immutable/assets/b.jpg 2x">');
	});

	it('leaves relative paths and other sites alone', () => {
		const html = '<img src="../app/immutable/assets/a.jpg"><img src="https://example.com/app/x.jpg">';
		expect(relativize(html, 1)).toBe(html);
	});
});
