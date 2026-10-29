import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SheetHeader from './SheetHeader.svelte';
import NomiHeader from './NomiHeader.svelte';

// Each design sheet opens with its DS Header (pen T4z7R on Build, the Chat one with a 40px title,
// and the Nomi sheet's title block beside its five principles).

const box = (element: Element) => element.getBoundingClientRect();

describe('SheetHeader', () => {
	it('stacks the eyebrow, title and an 880-wide lede 8px apart', async () => {
		await page.viewport(1512, 900);
		await render(SheetHeader, { eyebrow: 'DESIGN SYSTEM · BUILD', title: 'Everything between describe and a live URL', lede: 'Koda shows the plan.' });
		const title = page.getByRole('heading', { name: 'Everything between describe and a live URL' }).element();
		expect(box(title).top - box(page.getByText('DESIGN SYSTEM · BUILD').element()).bottom).toBe(8);
		expect(box(page.getByText('Koda shows the plan.').element()).width).toBe(880);
		expect(getComputedStyle(title).fontSize).toBe('28px');
	});

	it('sets the Chat sheet title at 40px', async () => {
		await render(SheetHeader, { eyebrow: 'DESIGN SYSTEM · CHAT RESPONSES', title: 'Everything Nomi can put in a reply', lede: 'Replies are prose first.', hero: true });
		const title = page.getByRole('heading', { name: 'Everything Nomi can put in a reply' }).element();
		expect(getComputedStyle(title).fontSize).toBe('40px');
		expect(box(title).height).toBe(52);
	});
});

describe('NomiHeader', () => {
	it('puts the title block beside the five principles, 335 tall with its divider', async () => {
		await page.viewport(1600, 900);
		const { container } = await render(NomiHeader);
		expect(box(container.querySelector('header')!).height).toBe(335);
		const principles = page.getByRole('list').element();
		expect(box(principles).width).toBe(560);
		expect([...principles.children].map((row) => box(row).height)).toEqual([59, 59, 59, 59, 59].map((height, index) => (index === 0 || index === 4 ? height - 1 : height)));
		await expect.element(page.getByText('NOMI DESIGN SYSTEM · v1.0 · 21 OCT 2026')).toBeVisible();
	});
});
