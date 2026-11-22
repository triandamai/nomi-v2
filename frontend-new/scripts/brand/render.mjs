// Draws Nomi's icons and the link-preview image into static/, from the crew's own shapes
// (src/lib/ds/agents/shapes.ts), so they can't drift from the mascots the app shows.
//
//   node scripts/brand/render.mjs
//
// favicon.svg      Nomi's shape and two-dot face; the eyes blink and glance (CSS inside the SVG,
//                  off under prefers-reduced-motion). Browsers that don't animate favicons show it still.
// favicon-48.png   The same, still, for browsers without SVG favicons; favicon.ico wraps it.
// apple-touch-icon.png, icon-192.png, icon-512.png   On the page colour, for home screens and the manifest.
// og-image.png     1200×630: the crew around Nomi, beside the wordmark and the landing page's line.

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';
import { createServer } from 'vite';

const root = fileURLToPath(new URL('../..', import.meta.url));
const out = (name) => `${root}static/${name}`;

// The page colour (--nm-bg, light) and the landing page's ink (--lp-text, --lp-text-2): a PNG
// can't read CSS variables. $lib/seo's test holds the page colour to the token.
const PAGE = '#F6F6F4';
const INK = '#1c1c1c';
const INK_2 = '#5c5c5c';
const OG = { width: 1200, height: 630 };
const ICON_SIZES = { 'favicon-48.png': 48, 'apple-touch-icon.png': 180, 'icon-192.png': 192, 'icon-512.png': 512 };
// On a home screen the shape sits inside the tile with room around it (the platforms round the corners).
const TILE_FILL = 0.78;
const BLINK_SECONDS = 5;
const GLANCE_SECONDS = 11;

const vite = await createServer({ root, configFile: false, server: { middlewareMode: true }, logLevel: 'error' });
const { CREW_LOOKS, FACE_COLOR, FACE_EYES, SHAPE_VIEW, TONE_STOPS, shapePath } = await vite.ssrLoadModule('/src/lib/ds/agents/shapes.ts');
const { CREW } = await vite.ssrLoadModule('/src/lib/ds/agents/crew.ts');
await vite.close();

/** A crew member's flat mascot as an SVG fragment, as agents/Mascot.svelte draws it. */
function mascot(agent, { id = agent, eyes = '' } = {}) {
	const look = CREW_LOOKS[agent];
	const stops = TONE_STOPS[look.tone];
	const paint = stops.map((color, i) => `<stop offset="${stops.length === 1 ? 0 : i / (stops.length - 1)}" stop-color="${color}"/>`).join('');
	const face = FACE_EYES.map((eye) => `<circle class="eye" cx="${eye.cx}" cy="${eye.cy}" r="${eye.r}" fill="${FACE_COLOR}"/>`).join('');
	return `<defs><linearGradient id="${id}" x1="0" y1="0" x2="1" y2="1">${paint}</linearGradient></defs><path d="${shapePath(look.shape)}" fill="url(#${id})"/><g class="eyes"${eyes}>${face}</g>`;
}

// The outline reaches 2..46 at its lobes: the favicon crops the view to the shape.
const FAVICON_VIEW = '2 2 44 44';
const favicon = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${FAVICON_VIEW}">
<style>
.eye { transform-box: fill-box; transform-origin: center; }
@media (prefers-reduced-motion: no-preference) {
	.eye { animation: blink ${BLINK_SECONDS}s ease-in-out infinite; }
	.eyes { animation: glance ${GLANCE_SECONDS}s ease-in-out infinite; }
}
@keyframes blink { 0%, 90%, 100% { transform: scaleY(1); } 94% { transform: scaleY(0.12); } }
@keyframes glance { 0%, 30%, 62%, 100% { transform: translate(0, 0); } 36%, 56% { transform: translate(2px, -0.5px); } }
</style>
${mascot('nomi', { id: 'nomi-paint' })}
</svg>
`;

/** A PNG wrapped as an ICO (one entry; every browser since IE takes PNG inside ICO). */
function ico(png, size) {
	const header = Buffer.alloc(6 + 16);
	header.writeUInt16LE(1, 2);
	header.writeUInt16LE(1, 4);
	header.writeUInt8(size, 6);
	header.writeUInt8(size, 7);
	header.writeUInt16LE(1, 10);
	header.writeUInt16LE(32, 12);
	header.writeUInt32LE(png.length, 14);
	header.writeUInt32LE(header.length, 18);
	return Buffer.concat([header, png]);
}

const font = async (path) => (await readFile(`${root}node_modules/${path}`)).toString('base64');
const geist = await font('@fontsource-variable/geist/files/geist-latin-wght-normal.woff2');

// The crew around Nomi: [agent, centre x, centre y, size, tilt in degrees], in the image's pixels.
const CLUSTER = [
	['koda', 640, 120, 96, -8],
	['dana', 1090, 110, 104, 10],
	['kala', 1120, 300, 84, -6],
	['rena', 1080, 500, 100, 8],
	['tara', 910, 565, 84, -10],
	['maya', 700, 545, 92, 6],
	['raga', 600, 400, 72, -4],
	['sora', 760, 70, 72, 12],
	['iris', 960, 60, 64, -12],
	['supervisor', 575, 545, 56, 0],
	['personality', 575, 255, 60, 14],
	['nomi', 870, 310, 300, 0],
];

const svgBox = (agent, size, extra = '') => `<svg width="${size}" height="${size}" viewBox="0 0 ${SHAPE_VIEW} ${SHAPE_VIEW}" ${extra}>${mascot(agent, { id: `${agent}-${size}` })}</svg>`;

const ogHtml = `<!doctype html><html><head><style>
@font-face { font-family: Geist; src: url(data:font/woff2;base64,${geist}) format('woff2'); font-weight: 100 900; }
* { margin: 0; box-sizing: border-box; }
body { width: ${OG.width}px; height: ${OG.height}px; background: ${PAGE}; font-family: Geist, sans-serif; position: relative; overflow: hidden; }
.copy { position: absolute; left: 80px; top: 0; bottom: 0; width: 470px; display: flex; flex-direction: column; justify-content: center; gap: 28px; }
h1 { font-size: 128px; font-weight: 800; letter-spacing: -0.05em; line-height: 0.9; color: ${INK}; }
p { font-size: 40px; font-weight: 500; letter-spacing: -0.02em; line-height: 1.2; color: ${INK_2}; }
.m { position: absolute; }
</style></head><body>
<div class="copy"><h1>nomi</h1><p>You talk.<br>Nomi’s crew gets it done.</p></div>
${CLUSTER.map(([agent, x, y, size, tilt]) => `<div class="m" style="left:${x - size / 2}px;top:${y - size / 2}px;transform:rotate(${tilt}deg)">${svgBox(agent, size)}</div>`).join('\n')}
</body></html>`;

const tile = (size) => `<!doctype html><html><head><style>* { margin: 0; } body { width: ${size}px; height: ${size}px; background: ${PAGE}; display: grid; place-items: center; }</style></head>
<body>${svgBox('nomi', Math.round(size * TILE_FILL))}</body></html>`;
const bare = (size) => `<!doctype html><html><head><style>* { margin: 0; } body { background: transparent; }</style></head>
<body><svg width="${size}" height="${size}" viewBox="${FAVICON_VIEW}">${mascot('nomi')}</svg></body></html>`;

const missing = CREW.map((member) => member.id).filter((id) => !CLUSTER.some(([agent]) => agent === id));
if (missing.length) throw new Error(`The preview image leaves out ${missing.join(', ')}`);

await mkdir(`${root}static`, { recursive: true });
await writeFile(out('favicon.svg'), favicon);

const browser = await chromium.launch();
async function shoot(html, width, height, file, transparent = false) {
	const page = await browser.newPage({ viewport: { width, height } });
	await page.setContent(html, { waitUntil: 'load' });
	await page.evaluate(() => document.fonts.ready);
	const png = await page.screenshot({ omitBackground: transparent, clip: { x: 0, y: 0, width, height } });
	await page.close();
	if (file) await writeFile(out(file), png);
	return png;
}

for (const [file, size] of Object.entries(ICON_SIZES)) {
	const png = await shoot(file === 'favicon-48.png' ? bare(size) : tile(size), size, size, file, file === 'favicon-48.png');
	if (file === 'favicon-48.png') await writeFile(out('favicon.ico'), ico(png, size));
}
await shoot(ogHtml, OG.width, OG.height, 'og-image.png');
await browser.close();
console.log(`static/: favicon.svg, favicon.ico, ${Object.keys(ICON_SIZES).join(', ')}, og-image.png (${missing.length ? '' : `${CREW.length} crew`})`);
