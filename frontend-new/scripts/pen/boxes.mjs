// An element's boxes in the running gallery, relative to it, a few levels down: the numbers to put
// next to the pen's frame bounds (pen/README.md) when compare.mjs points at a drift.
//
//   node scripts/pen/boxes.mjs <nomi|chat|build> '<css selector>' [depth=3] [match index=0] [--url …]
import { chromium } from 'playwright';
const args = process.argv.slice(2);
const base = args.includes('--url') ? args[args.indexOf('--url') + 1] : 'http://localhost:5175';
const [sheet, selector, depth = '3', index = '0'] = args.filter((arg, at) => !arg.startsWith('--') && args[at - 1] !== '--url');
const browser = await chromium.launch({ args: ['--font-render-hinting=none'] }); // as playwright.config.ts
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
await page.goto(`${base}/design-system/${sheet}?theme=dark`, { waitUntil: 'networkidle' });
await page.evaluate(() => document.fonts.ready);
const out = await page.evaluate(([selector, depth, index]) => {
	const root = document.querySelectorAll(selector)[index];
	if (!root) return 'not found';
	const r0 = root.getBoundingClientRect();
	const lines = [];
	const walk = (el, d) => {
		const r = el.getBoundingClientRect();
		const own = [...el.childNodes].filter((n) => n.nodeType === 3).map((n) => n.textContent.trim()).join(' ').slice(0, 30);
		lines.push(`${'  '.repeat(d)}${el.tagName.toLowerCase()} y${+(r.top - r0.top).toFixed(2)} x${+(r.left - r0.left).toFixed(2)} h${+r.height.toFixed(2)} w${+r.width.toFixed(2)}${own ? ` «${own}»` : ''}`);
		if (d < depth) for (const c of el.children) walk(c, d + 1);
	};
	walk(root, 0);
	return lines.join('\n');
}, [selector, Number(depth), Number(index)]);
console.log(out);
await browser.close();
