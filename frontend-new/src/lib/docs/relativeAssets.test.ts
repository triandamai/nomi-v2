import { describe, expect, it } from 'vitest';
import { relativize, trimFonts } from '../../../scripts/docs/relative-assets.mjs';

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

describe('the static docs’ fonts', () => {
	it('keeps the scripts the docs are written in, and drops the rest with their old-browser copies', () => {
		const face = (file: string, more = '') => `@font-face{font-family:Geist Variable;src:url(./${file})format("woff2-variations")${more}}`;
		const css = [
			face('geist-latin-wght-normal.a.woff2'),
			face('geist-latin-ext-wght-normal.b.woff2'),
			face('geist-cyrillic-wght-normal.c.woff2'),
			face('geist-mono-vietnamese-wght-normal.d.woff2'),
			face('geist-mono-symbols2-wght-normal.e.woff2'),
			'@font-face{font-family:Fraunces;src:url(./fraunces-latin-600-normal.f.woff2)format("woff2"),url(./fraunces-latin-600-normal.g.woff)format("woff")}',
			'.x{color:red}',
		].join('');
		const { css: trimmed, dropped } = trimFonts(css);
		expect(trimmed).toContain('geist-latin-wght-normal.a.woff2');
		expect(trimmed).toContain('geist-latin-ext-wght-normal.b.woff2');
		expect(trimmed).toContain('geist-mono-symbols2-wght-normal.e.woff2');
		expect(trimmed).toContain('fraunces-latin-600-normal.f.woff2');
		expect(trimmed).toContain('.x{color:red}');
		expect(trimmed).not.toMatch(/cyrillic|vietnamese|\.woff\)/);
		expect(dropped.sort()).toEqual(['fraunces-latin-600-normal.g.woff', 'geist-cyrillic-wght-normal.c.woff2', 'geist-mono-vietnamese-wght-normal.d.woff2']);
	});
});
