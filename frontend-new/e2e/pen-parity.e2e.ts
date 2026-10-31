import { readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';

// The design-system gallery has to sit on the pen: every section of the three sheets starts and
// ends where the pen's does, and every board is where the pen draws it, at its size. The numbers
// in pen/bounds.json come from the pen file itself (see pen/README.md for the snippet that
// exports them). A change that moves the gallery off the pen fails here; when the pen changes,
// re-export the bounds in the same PR.

type Board = [title: string, x: number, y: number, width: number, height: number];
type Sheet = { sections: [name: string, offset: number, height: number][]; boards: Record<string, Board[]> };
const pen: Record<string, Sheet> = JSON.parse(readFileSync(new URL('../pen/bounds.json', import.meta.url), 'utf8')).sheets;

const normalise = (name: string) => name.toLowerCase().replace(/[^a-z0-9]+/g, '');

for (const sheet of Object.keys(pen)) {
	test(`the ${sheet} sheet sits on the pen's bounds`, async ({ page }) => {
		await page.setViewportSize({ width: 1600, height: 900 });
		await page.goto(`/design-system/${sheet}?theme=dark`);
		await page.evaluate(() => document.fonts.ready);
		const ours = await page.evaluate(() => {
			const nav = document.querySelector('nav[aria-label="Design sheets"]')!;
			const parts = [...nav.parentElement!.children].filter((el) => el !== nav);
			const top = parts[0].getBoundingClientRect().top;
			const sections = parts.map((el) => [Math.round(el.getBoundingClientRect().top - top), Math.round(el.getBoundingClientRect().height)]);
			const boards: Record<string, [string, number, number, number, number][]> = {};
			for (const section of document.querySelectorAll('section[aria-label]')) {
				const s = section.getBoundingClientRect();
				boards[section.getAttribute('aria-label')!] = [...section.querySelectorAll('[data-board]')].map((b) => {
					const r = b.getBoundingClientRect();
					const title = b.firstElementChild?.firstElementChild?.textContent?.trim() ?? '';
					return [title, Math.round(r.left - s.left), Math.round(r.top - s.top), Math.round(r.width), Math.round(r.height)];
				});
			}
			return { sections, boards };
		});

		expect(ours.sections, 'one header and one block per pen section').toHaveLength(pen[sheet].sections.length);
		pen[sheet].sections.forEach(([name, offset, height], index) => {
			expect(ours.sections[index], `${sheet} / ${name}: offset from the header and height`).toEqual([offset, height]);
		});

		for (const [name, boards] of Object.entries(pen[sheet].boards)) {
			const key = Object.keys(ours.boards).find((label) => normalise(label) === normalise(name));
			expect(key, `${sheet} / ${name}: a gallery section`).toBeDefined();
			for (const [title, ...box] of boards) {
				const board = ours.boards[key!].find(([ourTitle]) => ourTitle.toLowerCase().startsWith(title.toLowerCase()));
				expect(board, `${sheet} / ${name} / ${title}: a board`).toBeDefined();
				expect(board!.slice(1), `${sheet} / ${name} / ${title}: x, y, width, height`).toEqual(box);
			}
		}
	});
}
