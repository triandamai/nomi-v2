import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import doc from './tokens.json';

// tokens.json is exported from the pen file (the design's own variables); tokens.css is what the
// app uses. If the design changes and the CSS doesn't follow, this fails and says which token.

type Token = { type: 'color' | 'number' | 'string'; light?: string; dark?: string; value?: string | number };
const tokens = doc.tokens as Record<string, Token>;

const css = readFileSync(new URL('./tokens.css', import.meta.url), 'utf8');
const declared = new Map([...css.matchAll(/--((?:nm|crew)-[a-z0-9-]+):\s*([^;]+);/g)].map((match) => [match[1], match[2].trim()]));

function expected(token: Token): string {
	if (token.type === 'number') return `${token.value}px`;
	if (token.type === 'color' && token.light) return `light-dark(${token.light}, ${token.dark})`;
	return String(token.value);
}

describe('design tokens', () => {
	it('declares every token from the design', () => {
		const missing = Object.keys(tokens).filter((name) => !declared.has(name));
		expect(missing).toEqual([]);
	});

	it('uses the design’s value for each token, in light and dark', () => {
		for (const [name, token] of Object.entries(tokens)) {
			const value = declared.get(name) ?? '';
			if (token.type === 'string') {
				// Font families: the design's family first, then fallbacks.
				expect(value, name).toMatch(new RegExp(`^"${token.value} Variable", "${token.value}"`));
			} else {
				expect(value.toLowerCase(), name).toBe(expected(token).toLowerCase());
			}
		}
	});

	it('adds no colour, size or space of its own', () => {
		const extra = [...declared.keys()].filter((name) => !(name in tokens));
		expect(extra).toEqual([]);
	});
});
