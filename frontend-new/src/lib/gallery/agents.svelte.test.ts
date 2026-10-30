import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AgentsSection from './AgentsSection.svelte';

const box = (element: Element) => element.getBoundingClientRect();
const style = (element: Element) => getComputedStyle(element);

// The pen's "Components / Crew Mascots" board orders the crew its own way.
const ORDER = ['Nomi', 'Dana', 'Kala', 'Maya', 'Tara', 'Raga', 'Personality', 'Supervisor', 'Rena', 'Koda'];

describe('Agents & app chrome section', () => {
	it('shows the crew mascots on a bordered board: 96px with names, then 32 and 20px rows', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(AgentsSection);
		const board = container.querySelector<HTMLElement>('[data-mascot-board]')!;
		expect(board.firstElementChild!.firstElementChild!.textContent).toBe('Crew mascots');
		await expect.element(page.getByText('1 colour + 1 silhouette per agent · flat · no head accessories')).toBeInTheDocument();
		expect(style(board).outlineWidth).toBe('1px');
		expect(style(board).outlineOffset).toBe('-1px');
		expect(style(board).borderTopLeftRadius).toBe('10px');
		expect(style(board).padding).toBe('40px');

		const rows = [...board.querySelectorAll<HTMLElement>('[data-size]')];
		expect(rows.map((row) => row.dataset.size)).toEqual(['96', '32', '20']);
		const [large, mid, small] = rows;
		const names = [...large.querySelectorAll('span')].map((name) => name.textContent);
		expect(names).toEqual(ORDER);
		for (const [row, px] of [[large, 96], [mid, 32], [small, 20]] as const) {
			const mascots = [...row.querySelectorAll('svg')];
			expect(mascots, `${px}`).toHaveLength(10);
			for (const mascot of mascots) expect(box(mascot).width).toBe(px);
			// Same order in every row: each mascot's body is drawn in that member's colour.
			expect(mascots.map((mascot) => mascot.querySelector('[data-part="body"]')!.getAttribute('style'))).toEqual(
				ORDER.map((name) => `fill: var(--crew-${name.toLowerCase()});`),
			);
		}
		for (const [row, label] of [[mid, '32px'], [small, '20px']] as const) {
			expect(row.firstElementChild!.textContent).toBe(label);
			expect(box(row.firstElementChild!).width).toBe(64);
		}
	});

	it('keeps the sidebar and settings nav either side of the board, as the pen does', async () => {
		const { container } = await render(AgentsSection);
		const library = container.querySelector<HTMLElement>('[data-mascot-board]')!.parentElement!;
		expect([...library.children].map((child) => (child as HTMLElement).dataset.library)).toEqual(['sidebar', 'mascots', 'settings']);
	});
});
