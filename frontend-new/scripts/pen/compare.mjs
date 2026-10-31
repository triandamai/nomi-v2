// Text-by-text comparison of one gallery section with the pen.
//
// Renders the pen's own HTML export of the section (pen/export/<sheet>--<slug>.html) in Chromium as
// the reference (local Geist, the pen's whole-pixel natural line heights, side strokes made
// layout-free as the pen draws them) and compares every text's position, relative to its section,
// with the same section of the running gallery. Each line marked ◀ is where the offset changes:
// the first element that drifts.
//
//   npm run dev                                   # the gallery on :5175
//   node scripts/pen/compare.mjs build workspace  # <sheet> <section slug>, see pen/export/
//   options: --all (every text), --theme light, --url http://localhost:4175
//
// The export is a good reference but not a perfect one: it collapses trailing spaces between inline
// runs, draws negative gaps and some fill widths differently. When a line looks wrong, check the
// pen's own frame bounds (pen/README.md) before changing the component.
import { chromium } from 'playwright';
import { readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
const args = process.argv.slice(2);
const [sheet, slug] = args;
if (!sheet || !slug) {
	console.log('usage: node scripts/pen/compare.mjs <nomi|chat|build> <section slug> [--all] [--theme light] [--url http://localhost:5175]');
	process.exit(1);
}
const all = args.includes('--all');
const theme = args.includes('--theme') ? args[args.indexOf('--theme') + 1] : 'dark';
const base = args.includes('--url') ? args[args.indexOf('--url') + 1] : 'http://localhost:5175';
const root = fileURLToPath(new URL('../../', import.meta.url));
const fonts = `${root}node_modules/@fontsource-variable`;
let html = readFileSync(`${root}pen/export/${sheet}--${slug}.html`, 'utf8');
html = html.replace(/<link[^>]*fonts\.g[^>]*>/g, '').replace(/<link\s+href="https:\/\/fonts[^>]*\/>/gs, '');
html = html.replace('</head>', `<style>
@font-face { font-family: Geist; src: url(file://${fonts}/geist/files/geist-latin-wght-normal.woff2) format('woff2'); font-weight: 100 900; }
@font-face { font-family: Geist; src: url(file://${fonts}/geist/files/geist-latin-ext-wght-normal.woff2) format('woff2'); font-weight: 100 900; unicode-range: U+0100-024F, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF; }
@font-face { font-family: Geist; src: url(file://${fonts}/geist/files/geist-latin-wght-italic.woff2) format('woff2'); font-weight: 100 900; font-style: italic; }
@font-face { font-family: "Geist Mono"; src: url(file://${fonts}/geist-mono/files/geist-mono-latin-wght-normal.woff2) format('woff2'); font-weight: 100 900; }
@font-face { font-family: "Geist Mono"; src: url(file://${fonts}/geist-mono/files/geist-mono-latin-ext-wght-normal.woff2) format('woff2'); font-weight: 100 900; unicode-range: U+0100-024F, U+1E00-1EFF, U+2020, U+20A0-20AB, U+20AD-20C0, U+2113, U+2C60-2C7F, U+A720-A7FF; }
</style></head>`);
const refFile = `${tmpdir()}/pen-ref-${sheet}--${slug}.html`;
writeFileSync(refFile, html);

const browser = await chromium.launch({ args: ['--font-render-hinting=none'] }); // as playwright.config.ts
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
await page.goto(`file://${refFile}`);
await page.evaluate(() => document.fonts.ready);
const collect = () => {
	const lineTop = (node) => {
		const range = document.createRange();
		range.selectNodeContents(node);
		const rect = range.getClientRects()[0];
		if (!rect) return null;
		const style = getComputedStyle(node.parentElement);
		const line = parseFloat(style.lineHeight) || rect.height;
		return { x: rect.left, y: rect.top - (line - rect.height) / 2, line };
	};
	return (root) => {
		const r0 = root.getBoundingClientRect();
		const rows = [];
		const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
		for (let node = walker.nextNode(); node; node = walker.nextNode()) {
			let text = node.textContent.replace(/\s+/g, ' ').trim();
			if (!text) continue;
			if (node.parentElement.closest('[hidden],[aria-hidden="true"] svg,dialog:not([open])')) continue;
			const style = getComputedStyle(node.parentElement);
			if (style.visibility === 'hidden' || style.display === 'none') continue;
			if (style.textTransform === 'uppercase') text = text.toUpperCase();
			const t = lineTop(node);
			if (!t) continue;
			rows.push([text.slice(0, 44), Math.round(t.x - r0.left), Math.round(t.y - r0.top), Math.round(t.line)]);
		}
		return { rows, w: Math.round(r0.width), h: Math.round(r0.height) };
	};
};
// Pen: whole-pixel natural line heights where the export says "normal".
const pen = await page.evaluate(`(() => {
	for (const el of document.querySelectorAll('[data-pencil-id]')) {
		const s = getComputedStyle(el);
		if (el.style.lineHeight === 'normal') el.style.lineHeight = Math.round(parseFloat(s.fontSize) * 1.295) + 'px';
		// Side strokes come out as real borders with content-box sizing and -0.5px margins; the pen
		// draws them inside the frame, so make them layout-free.
		if (el.style.borderStyle && el.style.borderStyle !== 'none') {
			const w = ['Top', 'Right', 'Bottom', 'Left'].map((side) => parseFloat(s['border' + side + 'Width']) || 0);
			const color = s.borderTopColor;
			const shadows = [];
			if (w[0]) shadows.push('inset 0 ' + w[0] + 'px 0 0 ' + color);
			if (w[2]) shadows.push('inset 0 -' + w[2] + 'px 0 0 ' + color);
			if (w[3]) shadows.push('inset ' + w[3] + 'px 0 0 0 ' + color);
			if (w[1]) shadows.push('inset -' + w[1] + 'px 0 0 0 ' + color);
			const mt = parseFloat(s.marginTop) || 0, mb = parseFloat(s.marginBottom) || 0, ml = parseFloat(s.marginLeft) || 0, mr = parseFloat(s.marginRight) || 0;
			if (el.style.boxSizing === 'content-box') {
				if (el.style.height.endsWith('px')) el.style.height = parseFloat(el.style.height) - mt - mb + 'px';
				if (el.style.width.endsWith('px')) el.style.width = parseFloat(el.style.width) - ml - mr + 'px';
				el.style.boxSizing = 'border-box';
			}
			el.style.borderWidth = '0';
			el.style.margin = '0';
			el.style.boxShadow = [shadows.join(', '), el.style.boxShadow].filter(Boolean).join(', ');
		}
	}
	const root = document.querySelector('body > [data-pencil-id]');
	// The pen's sections are 1472 wide (a 1600 sheet with 64 either side).
	if (root.style.width === '100%') root.style.width = '1472px';
	return (${collect.toString()})()(root);
})()`);
await page.goto(`${base}/design-system/${sheet}?theme=${theme}`, { waitUntil: 'networkidle' });
await page.evaluate(() => document.fonts.ready);
const title = readFileSync(`${root}pen/export/${sheet}--${slug}.html`, 'utf8').match(/data-pencil-name="Section \/ ([^"]+)"/)?.[1]?.replace(/&amp;/g, '&');
const ours = await page.evaluate(`(() => {
	const want = ${JSON.stringify(title ?? '')}.toLowerCase().replace(/[^a-z0-9]+/g, '');
	const root = [...document.querySelectorAll('section[aria-label], header')].find((s) => (s.getAttribute('aria-label') ?? '').toLowerCase().replace(/[^a-z0-9]+/g, '') === want) ?? (want ? null : document.querySelector('nav + header'));
	if (!root) return null;
	return (${collect.toString()})()(root);
})()`);
await browser.close();
if (!ours) { console.log(`no gallery section for "${title}"`); process.exit(1); }
console.log(`${sheet} / ${title}: pen ${pen.w}×${pen.h}  ours ${ours.w}×${ours.h}${pen.h === ours.h && pen.w === ours.w ? '  ✓' : ''}`);
let at = 0;
let last = null;
let missing = 0;
let shown = 0;
const used = new Set();
for (const [text, x, y, line] of pen.rows) {
	const key = text.slice(0, 22).toLowerCase();
	const i = ours.rows.findIndex((row, index) => index >= at && !used.has(index) && row[0].toLowerCase().startsWith(key));
	if (i < 0) {
		missing++;
		if (all || missing <= 12) console.log(`  ?? ${text.padEnd(44)} pen ${x},${y}`);
		continue;
	}
	used.add(i);
	at = i + 1;
	const [, ox, oy, oline] = ours.rows[i];
	const d = `${ox - x},${oy - y}`;
	const jump = d !== last;
	last = d;
	if (all || jump || oline !== line) {
		shown++;
		console.log(`  ${text.padEnd(44)} pen ${String(x).padStart(5)},${String(y).padStart(5)} lh${line}  ours ${String(ox).padStart(5)},${String(oy).padStart(5)} lh${oline}  Δ ${d}${jump ? ' ◀' : ''}`);
	}
}
console.log(`  (${pen.rows.length} pen texts, ${missing} not found in ours)`);
