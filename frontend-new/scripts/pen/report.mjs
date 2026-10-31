// Where the gallery stands against the pen, section by section and board by board.
//
// Reads pen/bounds.json (exported from the pen file, see pen/README.md) and measures the running
// gallery at 1600 wide: each section's offset from the sheet's header and its height, and each
// board's position and size inside its section. e2e/pen-parity.e2e.ts asserts the same numbers in
// CI; this prints them all, so you can see what moved.
//
//   npm run dev
//   node scripts/pen/report.mjs [nomi|chat|build ...] [--url http://localhost:5175]
import { chromium } from 'playwright';
import { readFileSync } from 'node:fs';

const args = process.argv.slice(2);
const base = args.includes('--url') ? args[args.indexOf('--url') + 1] : 'http://localhost:5175';
const pen = JSON.parse(readFileSync(new URL('../../pen/bounds.json', import.meta.url), 'utf8')).sheets;
const sheets = args.filter((arg, index) => !arg.startsWith('--') && args[index - 1] !== '--url');
const normalise = (name) => name.toLowerCase().replace(/[^a-z0-9]+/g, '');

const browser = await chromium.launch({ args: ['--font-render-hinting=none'] }); // as playwright.config.ts
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
let off = 0;
for (const sheet of sheets.length ? sheets : Object.keys(pen)) {
	await page.goto(`${base}/design-system/${sheet}?theme=dark`, { waitUntil: 'networkidle' });
	await page.evaluate(() => document.fonts.ready);
	const ours = await page.evaluate(() => {
		const nav = document.querySelector('nav[aria-label="Design sheets"]');
		const parts = [...nav.parentElement.children].filter((el) => el !== nav);
		const top = parts[0].getBoundingClientRect().top;
		const sections = parts.map((el) => [el.getAttribute('aria-label') ?? el.tagName.toLowerCase(), Math.round(el.getBoundingClientRect().top - top), Math.round(el.getBoundingClientRect().height)]);
		const boards = {};
		for (const section of document.querySelectorAll('section[aria-label]')) {
			const s = section.getBoundingClientRect();
			boards[section.getAttribute('aria-label')] = [...section.querySelectorAll('[data-board]')].map((b) => {
				const r = b.getBoundingClientRect();
				return [b.firstElementChild?.firstElementChild?.textContent?.trim() ?? '', Math.round(r.left - s.left), Math.round(r.top - s.top), Math.round(r.width), Math.round(r.height)];
			});
		}
		return { sections, boards };
	});
	console.log(`— ${sheet}`);
	pen[sheet].sections.forEach(([name, offset, height], index) => {
		const [, oo, oh] = ours.sections[index] ?? ['', NaN, NaN];
		const ok = oo === offset && oh === height;
		if (!ok) off++;
		console.log(`  ${ok ? '✓' : '✗'} ${name.padEnd(32)} pen ${String(offset).padStart(6)} +${String(height).padStart(5)}   ours ${String(oo).padStart(6)} +${String(oh).padStart(5)}`);
		const penBoards = pen[sheet].boards[name];
		if (!penBoards) return;
		const key = Object.keys(ours.boards).find((label) => normalise(label) === normalise(name));
		for (const [title, x, y, w, h] of penBoards) {
			const board = (ours.boards[key] ?? []).find(([t]) => t.toLowerCase().startsWith(title.toLowerCase()));
			const same = board && board[1] === x && board[2] === y && board[3] === w && board[4] === h;
			if (!same) {
				off++;
				console.log(`      ✗ ${title.padEnd(28)} pen ${x},${y} ${w}×${h}   ours ${board ? `${board[1]},${board[2]} ${board[3]}×${board[4]}` : 'missing'}`);
			}
		}
	});
}
await browser.close();
console.log(off ? `${off} off the pen` : 'all on the pen');
process.exitCode = off ? 1 : 0;
