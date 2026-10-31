import { describe, expect, it } from 'vitest';
import { parseExample } from './examples';

describe('parseExample', () => {
	it('names an example from its first comment and shows the rest as its code', () => {
		const raw = `<!-- Variants: one primary per view: inverse inside a panel. -->\n<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n</script>\n\n<Button label="Deploy" />\n`;
		expect(parseExample('./examples/button/01-variants.svelte', raw)).toEqual({
			title: 'Variants',
			description: 'One primary per view: inverse inside a panel.',
			code: `<script lang="ts">\n\timport Button from '$lib/ds/actions/Button.svelte';\n</script>\n\n<Button label="Deploy" />`,
		});
	});

	it('falls back to the file name', () => {
		expect(parseExample('./examples/button/02-with-icons.svelte', '<p>x</p>')).toEqual({ title: 'With icons', description: '', code: '<p>x</p>' });
	});
});
