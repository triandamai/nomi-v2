// After `vite build` of the static docs (DS_STATIC=1): an image a component imports is written into
// the prerendered page from the site's root ("/app/immutable/assets/…"), which 404s when the copy
// is served from a nested path (a share host's /shared/nomi-ds/). SvelteKit makes its own links
// relative; this does the same for those, by each page's depth in build-gallery/.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

/**
 * `html` with every `/app/…` reference (src, href, srcset) made relative to a page `depth` folders down.
 * @param {string} html
 * @param {number} depth
 * @returns {string}
 */
export function relativize(html, depth) {
	const up = depth === 0 ? './' : '../'.repeat(depth);
	return html.replace(/(^|[\s"',=(])\/app\//g, (/** @type {string} */ _, /** @type {string} */ before) => `${before}${up}app/`);
}

/**
 * @param {string} dir
 * @returns {string[]}
 */
function pages(dir) {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => (entry.isDirectory() ? pages(join(dir, entry.name)) : entry.name.endsWith('.html') ? [join(dir, entry.name)] : []));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
	const root = process.argv[2] ?? 'build-gallery';
	let changed = 0;
	for (const page of pages(root)) {
		const depth = relative(root, page).split(sep).length - 1;
		const html = readFileSync(page, 'utf8');
		const next = relativize(html, depth);
		if (next !== html) {
			writeFileSync(page, next);
			changed += 1;
		}
	}
	console.log(`relative-assets: ${changed} page(s) pointed at their own files`);
}
