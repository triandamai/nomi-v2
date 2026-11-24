// After `vite build` of the static docs (DS_STATIC=1): an image a component imports is written into
// the prerendered page from the site's root ("/app/immutable/assets/…"), which 404s when the copy
// is served from a nested path (a share host's /shared/nomi-ds/). SvelteKit makes its own links
// relative; this does the same for those, by each page's depth in build-gallery/.
// It also leaves out the font files the docs never use (Cyrillic and Vietnamese, and the .woff
// copies for browsers without woff2): a share host caps a site at about 500 files. The app keeps
// them all; only this static copy drops them.
import { readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { basename, join, relative, sep } from 'node:path';
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

const UNUSED_SCRIPTS = /-(cyrillic|cyrillic-ext|vietnamese)-/;

/**
 * `css` without the @font-face rules for scripts the docs aren't written in, and without the .woff
 * fallbacks beside a woff2; `dropped` names the files nothing points at any more.
 * @param {string} css
 * @returns {{ css: string; dropped: string[] }}
 */
export function trimFonts(css) {
	/** @type {string[]} */
	const dropped = [];
	const fileOf = (/** @type {string} */ url) => basename(url.replace(/^url\(["']?|["']?\)$/g, ''));
	let next = css.replace(/@font-face\{[^}]*\}/g, (rule) => {
		const urls = rule.match(/url\([^)]*\)/g) ?? [];
		if (urls.some((url) => UNUSED_SCRIPTS.test(url))) {
			dropped.push(...urls.map(fileOf));
			return '';
		}
		return rule;
	});
	next = next.replace(/,\s*url\(([^)]*\.woff)\)\s*format\("woff"\)/g, (_, url) => {
		dropped.push(fileOf(url));
		return '';
	});
	return { css: next, dropped };
}

/**
 * @param {string} dir
 * @param {string} ending
 * @returns {string[]}
 */
function filesEnding(dir, ending) {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => (entry.isDirectory() ? filesEnding(join(dir, entry.name), ending) : entry.name.endsWith(ending) ? [join(dir, entry.name)] : []));
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

	const dropped = new Set();
	for (const sheet of filesEnding(root, '.css')) {
		const trimmed = trimFonts(readFileSync(sheet, 'utf8'));
		writeFileSync(sheet, trimmed.css);
		trimmed.dropped.forEach((file) => dropped.add(file));
	}
	// Only a file no stylesheet or page still names goes.
	const named = [...filesEnding(root, '.css'), ...filesEnding(root, '.js'), ...pages(root)].map((file) => readFileSync(file, 'utf8')).join('\n');
	const gone = filesEnding(root, '').filter((file) => dropped.has(basename(file)) && !named.includes(basename(file)));
	gone.forEach((file) => rmSync(file));
	console.log(`relative-assets: ${gone.length} unused font file(s) left out`);
}
