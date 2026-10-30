import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import doc from '$lib/ds/tokens/tokens.json';
import ColourSection from './ColourSection.svelte';
import CrewColourSection from './CrewColourSection.svelte';
import TypeSection from './TypeSection.svelte';
import SpaceSection from './SpaceSection.svelte';

// The gallery rebuilds the design sheet "Design System / Nomi" from the same tokens the app uses,
// so it can be compared with the pen file side by side.

const tokens = doc.tokens as Record<string, { type: string; light?: string; dark?: string; value?: string | number }>;
const box = (element: Element) => element.getBoundingClientRect();
const style = (element: Element) => getComputedStyle(element);

// The colour tokens the sheet draws (Palette / dark and Palette / light), in its order.
const SHEET_COLOURS = [
	'nm-bg',
	'nm-surface',
	'nm-surface-2',
	'nm-border',
	'nm-border-strong',
	'nm-text',
	'nm-text-2',
	'nm-text-3',
	'nm-accent',
	'nm-accent-ink',
	'nm-accent-bg',
	'nm-danger',
	'nm-danger-bg',
	'nm-danger-border',
	'nm-warning',
	'nm-warning-bg',
	'nm-focus',
	'nm-overlay',
	'nm-shadow',
];

describe('Colour section', () => {
	it('shows the sheet’s 19 colour tokens in a dark and a light column, and nothing else', async () => {
		const { container } = await render(ColourSection);
		for (const name of SHEET_COLOURS) {
			expect(tokens[name]?.type, name).toBe('color');
			expect(page.getByText(name, { exact: true }).elements(), name).toHaveLength(2);
		}
		for (const theme of ['dark', 'light']) {
			const swatches = [...container.querySelectorAll(`[data-theme="${theme}"] [data-token]`)].map((swatch) => swatch.getAttribute('data-token'));
			expect(swatches).toEqual(SHEET_COLOURS);
		}
		// The pen has no Support group.
		expect(page.getByText('Support', { exact: true }).elements()).toHaveLength(0);
		expect(page.getByText('nm-heat-1', { exact: true }).elements()).toHaveLength(0);
	});

	it('paints each swatch in its own theme', async () => {
		const { container } = await render(ColourSection);
		const swatch = (theme: string) =>
			container.querySelector<HTMLElement>(`[data-theme="${theme}"] [data-token="nm-accent"]`)!;
		expect(style(swatch('dark')).backgroundColor).toBe('rgb(123, 227, 164)');
		expect(style(swatch('light')).backgroundColor).toBe('rgb(30, 154, 88)');
	});

	it('draws swatches, names and uses as the pen does', async () => {
		const { container } = await render(ColourSection);
		const swatch = container.querySelector<HTMLElement>('[data-theme="dark"] [data-token="nm-bg"]')!;
		expect([box(swatch).width, box(swatch).height]).toEqual([40, 24]);
		expect(swatch.classList).toContain('border-border-strong');
		const row = swatch.parentElement!;
		const [, name, use] = row.children;
		expect(box(name).width).toBe(140);
		expect(name.classList).toContain('text-text');
		expect(use.classList).toContain('text-text-3');
		// Theme label in primary text; groups stack at 6px.
		const label = container.querySelector('[data-theme="dark"]')!.firstElementChild!;
		expect(label.textContent).toBe('dark');
		expect(label.classList).toContain('text-text');
		expect(style(row.parentElement!).rowGap).toBe('6px');
	});
});

const CREW_IDS = ['nomi', 'koda', 'dana', 'kala', 'rena', 'tara', 'maya', 'raga', 'personality', 'supervisor'];

describe('Crew colour section', () => {
	it('gives each of the ten agents a card in its own background, all in one row', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(CrewColourSection);
		const cards = [...container.querySelectorAll<HTMLElement>('[data-crew]')];
		expect(cards.map((card) => card.dataset.crew)).toEqual(CREW_IDS);
		const top = box(cards[0]).top;
		for (const card of cards) {
			const id = card.dataset.crew!;
			expect(box(card).top, id).toBe(top);
			expect(card.classList, id).toContain(`bg-crew-${id}-bg`);
			expect(style(card).borderTopLeftRadius, id).toBe('12px');
			expect(style(card).padding, id).toBe('14px');
			expect(style(card).rowGap, id).toBe('10px');
			// A 20px mascot and a 14px dot in the member's colour.
			const mascot = card.querySelector('svg')!;
			expect([box(mascot).width, box(mascot).height], id).toEqual([20, 20]);
			const dot = card.querySelector<HTMLElement>('[data-part="dot"]')!;
			expect([box(dot).width, box(dot).height], id).toEqual([14, 14]);
			expect(dot.classList, id).toContain(`bg-crew-${id}`);
			// The name, 14 / 600, and the three token names, 10px mono, all in the ink colour.
			expect(card.classList, id).toContain(`text-crew-${id}-ink`);
			const name = card.querySelector<HTMLElement>('[data-part="name"]')!;
			expect(style(name).fontSize, id).toBe('14px');
			expect(style(name).fontWeight, id).toBe('600');
			expect(style(name).color, id).toBe(style(card).color);
			const names = [...card.querySelectorAll<HTMLElement>('[data-part="token"]')];
			expect(names.map((token) => token.textContent), id).toEqual([`crew-${id}`, `crew-${id}-bg`, `crew-${id}-ink`]);
			for (const token of names) {
				expect(style(token).fontSize).toBe('10px');
				expect(token.classList).toContain('font-mono');
				expect(style(token).color).toBe(style(card).color);
			}
			for (const token of [`crew-${id}`, `crew-${id}-bg`, `crew-${id}-ink`]) expect(tokens[token], token).toBeDefined();
		}
		expect(page.getByText('Personality', { exact: true }).elements()).toHaveLength(1);
	});

	it('puts the rules in two bordered boxes', async () => {
		const { container } = await render(CrewColourSection);
		const rules = [...container.querySelectorAll<HTMLElement>('[data-rules]')];
		expect(rules.map((rule) => rule.firstElementChild!.textContent)).toEqual(['Use it for', 'Never for']);
		for (const rule of rules) {
			expect(style(rule).borderTopWidth).toBe('1px');
			expect(style(rule).borderTopLeftRadius).toBe('12px');
			expect(style(rule).padding).toBe('16px');
			expect(style(rule).rowGap).toBe('8px');
			for (const item of rule.querySelectorAll('li')) expect(style(item).fontSize).toBe('13px');
		}
		await expect.element(page.getByText('· Buttons, links, focus or live state — that\'s nm-accent')).toBeInTheDocument();
	});
});

describe('Type section', () => {
	it('shows the five sizes with their use', async () => {
		await render(TypeSection);
		for (const [name, px] of [
			['nm-size-display', 28],
			['nm-size-body', 15],
			['nm-size-ui', 13],
			['nm-size-caption', 12],
			['nm-size-label', 11],
		] as const) {
			expect(tokens[name].value).toBe(px);
			expect(page.getByText(name, { exact: true }).elements().length, name).toBeGreaterThan(0);
		}
		const greeting = page.getByText('Morning, Mario.').element();
		expect(style(greeting).fontSize).toBe('28px');
		expect(style(greeting).fontWeight).toBe('500');
		const label = page.getByText("What's in this project").element();
		expect(style(label).textTransform).toBe('uppercase');
	});

	it('lays the styles out as the pen’s bordered table', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(TypeSection);
		const table = container.querySelector<HTMLElement>('[data-type-table]')!;
		expect(style(table).borderTopWidth).toBe('1px');
		expect(style(table).borderTopLeftRadius).toBe('10px');
		expect(style(table).overflow).toBe('hidden');
		const rows = [...table.children] as HTMLElement[];
		expect(rows).toHaveLength(6);
		for (const [index, row] of rows.entries()) {
			expect(style(row).paddingLeft).toBe('20px');
			expect(style(row).paddingTop).toBe('16px');
			expect(style(row).alignItems).toBe('center');
			const [meta, sample, use] = row.children;
			expect(box(meta).width).toBe(200);
			expect(style(meta).rowGap).toBe('2px');
			// Samples in primary text; the section label keeps its own text-3.
			expect(sample.classList).toContain(index === 5 ? 'text-text-3' : 'text-text');
			expect(box(use).width).toBe(220);
			expect(use.classList).toContain('text-text-3');
		}
	});
});

// The pen's icon grid (Block / Iconography), row by row.
const ICONS = [
	'house', 'message-square', 'folder', 'wallet', 'bell', 'git-fork', 'inbox', 'search',
	'plus', 'arrow-up', 'mic', 'paperclip', 'rotate-ccw', 'download', 'git-branch', 'globe',
	'database', 'rocket', 'circle-alert', 'check', 'x', 'settings', 'users', 'arrow-up-right',
];

describe('Space, radius and icons section', () => {
	it('is one section of three bordered blocks', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(SpaceSection);
		const section = container.querySelector('section[aria-label="Space, radius & icons"]')!;
		const blocks = [...section.children] as HTMLElement[];
		expect(blocks.map((block) => block.firstElementChild!.textContent)).toEqual(['Spacing', 'Radius & depth', 'Iconography']);
		for (const block of blocks) {
			expect(style(block).borderTopWidth).toBe('1px');
			expect(style(block).padding).toBe('24px');
		}
		expect(box(blocks[1]).width).toBe(360);
		expect(box(blocks[2]).width).toBe(360);
	});

	it('draws each spacing step as a bar three times its value', async () => {
		const { container } = await render(SpaceSection);
		const steps: [string, number][] = [['2xs', 2], ['xs', 4], ['sm', 6], ['md', 8], ['lg', 12], ['xl', 16], ['2xl', 20], ['3xl', 24], ['4xl', 32], ['5xl', 48]];
		let previous: number | undefined;
		for (const [step, px] of steps) {
			const name = page.getByText(`space-${step}`, { exact: true }).element();
			expect(box(name).width, step).toBe(80);
			const row = name.parentElement!;
			expect(style(row).columnGap, step).toBe('12px');
			const bar = row.children[1];
			expect([box(bar).width, box(bar).height], step).toEqual([px * 3, 8]);
			if (previous !== undefined) expect(box(row).top - previous, step).toBe(30);
			previous = box(row).top;
		}
		expect(container.querySelectorAll('[data-space-bar]')).toHaveLength(10);
	});

	it('shows both radii at 148 × 56, the flat and floating depths, the icon grid and the icon sizes', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(SpaceSection);
		for (const text of ['radius-sm · 6px', 'radius-md · 10px', 'Flat + border', 'Floating']) {
			const sample = page.getByText(text, { exact: true }).element().previousElementSibling!;
			// 148 in the pen, whose strokes take no room; the block's 1px CSS border takes one each side.
			expect([box(sample).width, box(sample).height], text).toEqual([147, 56]);
		}
		const tiles = [...container.querySelectorAll<HTMLElement>('[data-icon-tile]')];
		expect(tiles.map((tile) => tile.dataset.iconTile)).toEqual(ICONS);
		for (const tile of tiles) {
			expect([box(tile).width, box(tile).height]).toEqual([32, 32]);
			expect(tile.classList).toContain('bg-surface');
			expect(tile.classList).toContain('text-text-2');
			const svg = tile.querySelector('svg')!;
			expect(svg.classList, tile.dataset.iconTile).toContain(`lucide-${tile.dataset.iconTile}`);
			expect(box(svg).width).toBe(16);
		}
		for (const size of [16, 14, 12]) {
			const caption = page.getByText(`${size}px`, { exact: true }).element();
			const icon = caption.previousElementSibling!.querySelector('svg') ?? caption.previousElementSibling!;
			expect(icon.classList).toContain('lucide-folder');
			expect(box(icon).width).toBe(size);
			expect(caption.parentElement!.classList).toContain('text-text');
		}
	});
});
