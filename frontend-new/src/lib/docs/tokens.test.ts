import { describe, expect, it } from 'vitest';
import { tokenGroups } from './tokens';

describe('tokenGroups', () => {
	const tokens = {
		'nm-bg': { type: 'color', light: '#F6F6F4', dark: '#0A0A0B' },
		'nm-accent': { type: 'color', light: '#1E9A58', dark: '#7BE3A4' },
		'crew-koda': { type: 'color', light: '#2563EB', dark: '#7AA7FF' },
		'nm-space-md': { type: 'number', value: 8 },
		'nm-space-2xs': { type: 'number', value: 2 },
		'nm-size-ui': { type: 'number', value: 13 },
		'nm-mono': { type: 'string', value: 'Geist Mono' },
		'nm-radius-sm': { type: 'number', value: 6 },
	};
	const css = `@theme inline {\n\t--color-bg: var(--nm-bg);\n\t--color-accent: var(--nm-accent);\n\t--color-crew-koda: var(--crew-koda);\n\t--font-mono: var(--nm-mono);\n\t--text-ui: var(--nm-size-ui);\n\t--radius-sm: var(--nm-radius-sm);\n}`;
	const groups = tokenGroups(tokens, css);

	it('groups the tokens as the pen does, in its order', () => {
		expect(groups.map((group) => [group.title, group.rows.map((row) => row.name)])).toEqual([
			['Surfaces & text', ['nm-bg']],
			['Accent & signals', ['nm-accent']],
			['Crew', ['crew-koda']],
			['Type', ['nm-size-ui', 'nm-mono']],
			['Space', ['nm-space-md', 'nm-space-2xs']],
			['Radius', ['nm-radius-sm']],
		]);
	});

	it('names the CSS variable and the classes that use each token', () => {
		const rows = Object.fromEntries(groups.flatMap((group) => group.rows).map((row) => [row.name, row]));
		expect(rows['nm-bg']).toEqual({ name: 'nm-bg', cssVar: '--nm-bg', classes: ['bg-bg', 'text-bg', 'border-bg'], light: '#F6F6F4', dark: '#0A0A0B' });
		expect(rows['crew-koda'].classes).toEqual(['bg-crew-koda', 'text-crew-koda', 'border-crew-koda']);
		expect(rows['nm-mono']).toMatchObject({ classes: ['font-mono'], value: 'Geist Mono' });
		expect(rows['nm-size-ui']).toMatchObject({ classes: ['text-ui'], value: '13px' });
		expect(rows['nm-radius-sm']).toMatchObject({ classes: ['rounded-sm'], value: '6px' });
		expect(rows['nm-space-md']).toMatchObject({ classes: ['p-2', 'gap-2'], value: '8px' });
		expect(rows['nm-space-2xs']).toMatchObject({ classes: ['p-0.5', 'gap-0.5'], value: '2px' });
	});
});
