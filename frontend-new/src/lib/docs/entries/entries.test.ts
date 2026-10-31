import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { componentDocs } from './index';

const ds = fileURLToPath(new URL('../../ds/', import.meta.url));

describe('component docs', () => {
	const docs = Object.values(componentDocs);

	it('describe components that exist, and point only at components that exist', () => {
		expect(docs.filter((doc) => !existsSync(ds + doc.file)).map((doc) => doc.file)).toEqual([]);
		const broken = docs.flatMap((doc) => (doc.related ?? []).filter((file) => !existsSync(ds + file)).map((file) => `${doc.file} → ${file}`));
		expect(broken).toEqual([]);
	});

	it('say what each one is for, in a sentence or two', () => {
		expect(docs.filter((doc) => !doc.summary.trim() || doc.summary.length > 320).map((doc) => doc.file)).toEqual([]);
	});
});
