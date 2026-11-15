import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AsksInLine from './AsksInLine.svelte';

// Flow / Build — Invite & build together (TnwQT) → step 9 "Two asks at once": the line of asks at
// the end of a project's chat (Chat / Request Queue t2BII, 480 wide in the flow's thread).

const mario = { name: 'Mario Prasetyo', initials: 'MP' };
const sari = { name: 'Sari Wulandari', initials: 'SW', peer: 1 as const };
const box = (element: Element) => element.getBoundingClientRect();

beforeEach(async () => {
	await page.viewport(1280, 800);
});

describe('Asks in line', () => {
	it('lays out as the pen draws it: a 40 header, 53 per ask, a 48 footer', async () => {
		const { container } = await render(AsksInLine, {
			asks: [
				{ text: 'Make the Pre-order button green', who: mario, state: 'next', meta: 'Mario Prasetyo · next' },
				{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'queued', meta: 'Sari Wulandari · waiting' },
			],
			first: 1,
			onfirst: () => {},
		});
		const card = container.querySelector('section')!;
		card.style.width = '480px';
		expect(box(card.querySelector('header')!).height).toBe(40);
		for (const row of card.querySelectorAll('li')) expect(box(row).height).toBe(53);
		expect(box(card.querySelector('footer')!).height).toBe(48);
	});

	it('names the line, says who asked what and where it is: Koda building one, the next one tagged', async () => {
		await render(AsksInLine, {
			asks: [
				{ text: 'Make the Pre-order button green', who: mario, state: 'running', meta: 'Mario Prasetyo · running' },
				{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'next', meta: 'Sari Wulandari · next' },
			],
		});
		await expect.element(page.getByRole('region', { name: '2 asks in line' })).toBeVisible();
		await expect.element(page.getByText('Koda is building')).toBeVisible();
		await expect.element(page.getByText('Next', { exact: true })).toBeVisible();
		await expect.element(page.getByText('Sari Wulandari · next')).toBeVisible();
		// Nothing can be moved: no button.
		expect(page.getByRole('button').elements()).toHaveLength(0);
	});

	it('lets a teammate’s ask waiting behind yours go first', async () => {
		const onfirst = vi.fn();
		await render(AsksInLine, {
			asks: [
				{ text: 'Rename the shop', who: sari, state: 'running' },
				{ text: 'Make the Pre-order button green', who: mario, state: 'next' },
				{ text: 'Add a Makanan section', who: sari, state: 'queued' },
			],
			first: 2,
			onfirst,
		});
		await page.getByRole('button', { name: "Do Sari's first" }).click();
		expect(onfirst).toHaveBeenCalledWith(2);
	});
});
