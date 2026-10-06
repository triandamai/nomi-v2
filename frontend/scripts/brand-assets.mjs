// Generates Nomi's icons and link-preview image into static/ from the same shape geometry the
// app draws (src/lib/components/m3/shapes.ts), so the brand mark never drifts from the UI.
//
//   node --experimental-strip-types scripts/brand-assets.mjs
//
// Writes favicon.svg, favicon-48.png, apple-touch-icon.png, icon-192.png, icon-512.png and
// og-image.png. Rendering uses the Playwright Chromium; fonts come from Google Fonts via curl.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from 'playwright';
import { GRADIENT_STOPS, agentLook, shapePath } from '../src/lib/components/m3/shapes.ts';

const STATIC = new URL('../static/', import.meta.url).pathname;
const SURFACE = '#f4f7f2';
const ON_SURFACE = '#17201b';
const ON_SURFACE_VARIANT = '#46534b';
const EYES = '#062019';

/** One agent's shape as SVG markup in a 48×48 box, optionally with Nomi's face. */
function shape(agent, id, { face = false } = {}) {
	const look = agentLook(agent);
	const stops = GRADIENT_STOPS[look.tone];
	const gradient = stops.map((c, i) => `<stop offset="${stops.length === 1 ? 0 : i / (stops.length - 1)}" stop-color="${c}"/>`).join('');
	const eyes = face ? `<circle cx="19" cy="22" r="2.6" fill="${EYES}"/><circle cx="29" cy="22" r="2.6" fill="${EYES}"/>` : '';
	return `<defs><linearGradient id="${id}" x1="0" y1="0" x2="1" y2="1">${gradient}</linearGradient></defs><path d="${shapePath(look.shape)}" fill="url(#${id})"/>${eyes}`;
}

const nomi = (id = 'g') => shape('nomi', id, { face: true });

// Favicon: the shape fills the box; eyes stay legible at 16px.
const faviconSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="2 2 44 44">${nomi()}</svg>\n`;
writeFileSync(join(STATIC, 'favicon.svg'), faviconSvg);

/** Latin subset of a Google Font, as a data URL (the share image has no network of its own). */
function font(query) {
	const ua = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120 Safari/537.36';
	const css = execFileSync('curl', ['-sS', '-A', ua, `https://fonts.googleapis.com/css2?family=${query}&display=swap`], { encoding: 'utf8' });
	const urls = [...css.matchAll(/url\((https:[^)]+)\)/g)].map((m) => m[1]);
	const dir = mkdtempSync(join(tmpdir(), 'nomi-font-'));
	const file = join(dir, 'font.woff2');
	execFileSync('curl', ['-sS', '-o', file, urls.at(-1)]);
	return `data:font/woff2;base64,${readFileSync(file).toString('base64')}`;
}

const brandFont = font('Bricolage+Grotesque:opsz,wght@12..96,800');
const bodyFont = font('Instrument+Sans:wght@500');

const base = `<style>
	@font-face { font-family: Brand; src: url(${brandFont}) format('woff2'); font-weight: 800; }
	@font-face { font-family: Body; src: url(${bodyFont}) format('woff2'); font-weight: 500; }
	* { margin: 0; box-sizing: border-box; }
	html, body { background: transparent; }
</style>`;

// App icons: the mark on Nomi's surface, with room around it for launchers that round or crop.
const iconPage = (size, padding) => `<!doctype html><html><head>${base}<style>
	body { width: ${size}px; height: ${size}px; display: grid; place-items: center; background: ${SURFACE}; }
	svg { width: ${size - padding * 2}px; height: ${size - padding * 2}px; }
</style></head><body><svg viewBox="0 0 48 48">${nomi()}</svg></body></html>`;

// Link preview (1200×630): the mark with three of the crew around it, the wordmark and the line
// from the sign-in stage.
const crew = [
	{ agent: 'money', size: 92, x: 70, y: 70 },
	{ agent: 'planning', size: 120, x: 395, y: 410 },
	{ agent: 'reminders', size: 70, x: 455, y: 95 },
];
const ogPage = `<!doctype html><html><head>${base}<style>
	body { width: 1200px; height: 630px; position: relative; overflow: hidden; background: ${SURFACE}; }
	.halo { position: absolute; left: 70px; top: 85px; width: 460px; height: 460px; border-radius: 50%;
		background: radial-gradient(circle, rgba(91, 224, 143, 0.28), rgba(91, 224, 143, 0) 70%); }
	.mark { position: absolute; left: 115px; top: 130px; width: 370px; height: 370px; }
	.crew { position: absolute; }
	.text { position: absolute; left: 610px; top: 0; bottom: 0; right: 70px; display: flex; flex-direction: column; justify-content: center; gap: 22px; }
	.word { font-family: Brand; font-weight: 800; font-size: 168px; line-height: 0.9; letter-spacing: -0.04em; color: ${ON_SURFACE}; }
	.line { font-family: Body; font-weight: 500; font-size: 40px; line-height: 1.25; color: ${ON_SURFACE_VARIANT}; max-width: 15ch; }
</style></head><body>
	<div class="halo"></div>
	<svg class="mark" viewBox="0 0 48 48">${nomi('nomi')}</svg>
	${crew.map((c, i) => `<svg class="crew" style="left:${c.x}px;top:${c.y}px;width:${c.size}px;height:${c.size}px" viewBox="0 0 48 48">${shape(c.agent, `c${i}`)}</svg>`).join('')}
	<div class="text"><div class="word">nomi</div><div class="line">A small crew of agents that remembers you.</div></div>
</body></html>`;

const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/opt/pw-browsers/chromium' });
async function render(html, width, height, file, transparent = false) {
	const page = await browser.newPage({ viewport: { width, height } });
	await page.setContent(html);
	await page.evaluate(() => document.fonts.ready);
	await page.screenshot({ path: join(STATIC, file), omitBackground: transparent });
	await page.close();
}

await render(`<!doctype html><html><head>${base}<style>body{width:48px;height:48px}svg{width:48px;height:48px;display:block}</style></head><body>${faviconSvg}</body></html>`, 48, 48, 'favicon-48.png', true);
await render(iconPage(180, 22), 180, 180, 'apple-touch-icon.png');
await render(iconPage(192, 24), 192, 192, 'icon-192.png');
await render(iconPage(512, 64), 512, 512, 'icon-512.png');
await render(ogPage, 1200, 630, 'og-image.png');
await browser.close();
console.log('wrote favicon.svg, favicon-48.png, apple-touch-icon.png, icon-192.png, icon-512.png, og-image.png');
