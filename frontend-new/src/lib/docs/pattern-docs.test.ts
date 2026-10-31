import { describe, expect, it } from 'vitest';
import { PATTERNS } from './patterns';
import { patternDocs } from './pattern-docs';
import type { PatternDoc } from './types';

// Every section of the Chat responses and Build sheets is a pattern page, and each page opens with
// its doc. The docs live one sheet per file in pattern-docs/; read them raw too, so a doc written
// twice (which the index would silently collapse) is caught.
const files = import.meta.glob<PatternDoc[]>(['./pattern-docs/*.ts', '!./pattern-docs/index.ts'], { eager: true, import: 'default' });
const docs = Object.values(files).flat();
const key = (item: { sheet: string; id: string }) => `${item.sheet}/${item.id}`;

describe('pattern docs', () => {
	it('cover every pattern on the sheets', () => {
		expect(PATTERNS.map(key).filter((k) => !patternDocs[k])).toEqual([]);
	});

	it('describe only patterns that exist, once each', () => {
		const patterns = new Set(PATTERNS.map(key));
		expect(docs.map(key).filter((k) => !patterns.has(k))).toEqual([]);
		expect(docs.map(key).filter((k, i, all) => all.indexOf(k) !== i)).toEqual([]);
	});

	it('open with a short summary of the job', () => {
		expect(docs.filter((doc) => !doc.summary.trim() || doc.summary.length > 360).map(key)).toEqual([]);
	});
});
