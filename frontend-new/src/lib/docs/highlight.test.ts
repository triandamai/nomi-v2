import { describe, expect, it } from 'vitest';
import { highlight } from './highlight';

const kinds = (code: string) => highlight(code).map((line) => line.map((token) => [token.text, token.kind ?? '']));

describe('highlight', () => {
	it('colours a Svelte example with the code tokens', () => {
		expect(kinds(`<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n</script>`)).toEqual([
			[['<script', 'punct'], [' ', ''], ['lang', 'fn'], ['=', 'punct'], ['"ts"', 'string'], ['>', 'punct']],
			[['\t', ''], ['import', 'keyword'], [' Button ', ''], ['from', 'keyword'], [' ', ''], ["'$lib/ds/actions/Button.svelte'", 'string'], [';', 'punct']],
			[['</script>', 'punct']],
		]);
	});

	it('marks components, attributes, numbers, comments and blocks', () => {
		expect(kinds(`<Button label="Deploy" size={14} />`)).toEqual([
			[['<Button', 'type'], [' ', ''], ['label', 'fn'], ['=', 'punct'], ['"Deploy"', 'string'], [' ', ''], ['size', 'fn'], ['=', 'punct'], ['{', 'punct'], ['14', 'number'], ['}', 'punct'], [' ', ''], ['/>', 'punct']],
		]);
		expect(kinds('<!-- a note -->')).toEqual([[['<!-- a note -->', 'comment']]]);
		expect(kinds('{#each items as item (item.id)}')[0][0]).toEqual(['{#each', 'keyword']);
		expect(kinds('// later')).toEqual([[['// later', 'comment']]]);
	});

	it('reads plain TypeScript too', () => {
		expect(highlight("import Button from '$lib/ds/actions/Button.svelte';", 'ts')[0].map((token) => token.kind ?? '')).toEqual(['keyword', '', 'keyword', '', 'string', 'punct']);
	});

	it('leaves a blank line empty', () => {
		expect(highlight('a\n\nb')[1]).toEqual([]);
	});
});
