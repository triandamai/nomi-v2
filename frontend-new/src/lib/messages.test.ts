import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// Every string the person reads comes from messages/{en,id}.json. A key that exists in one
// language only shows the raw key in the other, and a placeholder missing from a translation
// silently drops a value (a name, an amount) from the sentence.

function load(locale: string): Record<string, string> {
	const raw = JSON.parse(readFileSync(new URL(`../../messages/${locale}.json`, import.meta.url), 'utf8')) as Record<string, unknown>;
	return Object.fromEntries(Object.entries(raw).filter(([key]) => key !== '$schema')) as Record<string, string>;
}

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();

describe('messages', () => {
	const en = load('en');
	const id = load('id');

	it('has the same keys in English and Indonesian', () => {
		expect(Object.keys(en).length).toBeGreaterThan(0);
		expect(Object.keys(id).sort()).toEqual(Object.keys(en).sort());
	});

	it('translates every string', () => {
		for (const [key, text] of Object.entries(id)) {
			expect(text.trim(), key).not.toBe('');
		}
	});

	it('keeps every placeholder in the translation', () => {
		for (const [key, text] of Object.entries(en)) {
			expect(placeholders(id[key] ?? ''), key).toEqual(placeholders(text));
		}
	});
});
