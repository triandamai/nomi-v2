import { existsSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { FACE_EYES, shapePath } from '$lib/ds/agents/shapes';
import { SHARE_IMAGE, THEME_COLOR, absoluteUrl, shareMeta } from './seo';

const file = (path: string) => new URL(`../../../${path}`, import.meta.url);
const text = (path: string) => readFileSync(file(path), 'utf8');

/** A PNG's width and height, from its IHDR chunk. */
function pngSize(path: string) {
	const bytes = readFileSync(file(path));
	return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

const meta = shareMeta({
	origin: 'https://nomi.build',
	pathname: '/',
	title: 'Nomi: you talk, the crew gets it done',
	description: 'Nomi is one chat with a crew of AI agents.',
	siteName: 'Nomi',
	imageAlt: 'Nomi and the crew',
	locale: 'en',
});
const content = (key: string) => meta.find((tag) => tag.property === key || tag.name === key)?.content;

describe('share meta', () => {
	it('builds absolute addresses from the origin it is given, never a fixed host', () => {
		expect(absoluteUrl('https://nomi.build', '/og-image.png')).toBe('https://nomi.build/og-image.png');
		expect(absoluteUrl('http://localhost:5175/', '/og-image.png')).toBe('http://localhost:5175/og-image.png');
		const staging = shareMeta({ origin: 'https://staging.nomi.build', pathname: '/sign-in', title: 't', description: 'd', siteName: 's', imageAlt: 'a', locale: 'id' });
		expect(staging.find((tag) => tag.property === 'og:url')?.content).toBe('https://staging.nomi.build/sign-in');
		expect(staging.find((tag) => tag.property === 'og:image')?.content).toBe('https://staging.nomi.build/og-image.png');
	});

	it('has the description, Open Graph and a large Twitter card', () => {
		expect(content('description')).toBe('Nomi is one chat with a crew of AI agents.');
		expect(content('og:type')).toBe('website');
		expect(content('og:site_name')).toBe('Nomi');
		expect(content('og:title')).toBe('Nomi: you talk, the crew gets it done');
		expect(content('og:description')).toBe('Nomi is one chat with a crew of AI agents.');
		expect(content('og:url')).toBe('https://nomi.build/');
		expect(content('og:image')).toBe('https://nomi.build/og-image.png');
		expect(content('og:image:width')).toBe('1200');
		expect(content('og:image:height')).toBe('630');
		expect(content('og:image:type')).toBe('image/png');
		expect(content('og:image:alt')).toBe('Nomi and the crew');
		expect(content('og:locale')).toBe('en_US');
		expect(content('twitter:card')).toBe('summary_large_image');
		expect(content('twitter:title')).toBe('Nomi: you talk, the crew gets it done');
		expect(content('twitter:image')).toBe('https://nomi.build/og-image.png');
	});

	it('names the Indonesian locale for an Indonesian page', () => {
		const id = shareMeta({ origin: 'https://nomi.build', pathname: '/', title: 't', description: 'd', siteName: 's', imageAlt: 'a', locale: 'id' });
		expect(id.find((tag) => tag.property === 'og:locale')?.content).toBe('id_ID');
	});

	it('points at an image that is there, at the size it says', () => {
		expect(pngSize(`static${SHARE_IMAGE.path}`)).toEqual({ width: SHARE_IMAGE.width, height: SHARE_IMAGE.height });
	});

	it('colours the browser bar with the page token in each theme', () => {
		const bg = text('src/lib/ds/tokens/tokens.css').match(/--nm-bg:\s*light-dark\((#[0-9a-f]+),\s*(#[0-9a-f]+)\)/i);
		expect(bg?.slice(1, 3).map((hex) => hex.toLowerCase())).toEqual([THEME_COLOR.light.toLowerCase(), THEME_COLOR.dark.toLowerCase()]);
		const manifest = JSON.parse(text('static/site.webmanifest'));
		expect(manifest.theme_color.toLowerCase()).toBe(THEME_COLOR.light.toLowerCase());
		expect(manifest.background_color.toLowerCase()).toBe(THEME_COLOR.light.toLowerCase());
	});
});

describe('icons', () => {
	it('draws the favicon as Nomi’s own shape and face (scripts/brand/render.mjs)', () => {
		const svg = text('static/favicon.svg');
		expect(svg).toContain(`d="${shapePath('cookie9')}"`);
		for (const eye of FACE_EYES) expect(svg).toContain(`cx="${eye.cx}" cy="${eye.cy}" r="${eye.r}"`);
	});

	it('animates the eyes only when the person hasn’t asked for less motion', () => {
		const svg = text('static/favicon.svg');
		const motion = svg.indexOf('@media (prefers-reduced-motion: no-preference)');
		expect(motion).toBeGreaterThan(-1);
		expect(svg.indexOf('animation:')).toBeGreaterThan(motion);
	});

	it('has the still fallbacks the head and the manifest name', () => {
		expect(pngSize('static/favicon-48.png')).toEqual({ width: 48, height: 48 });
		expect(pngSize('static/apple-touch-icon.png')).toEqual({ width: 180, height: 180 });
		expect(existsSync(file('static/favicon.ico'))).toBe(true);
		const manifest = JSON.parse(text('static/site.webmanifest'));
		for (const icon of manifest.icons) {
			const [width, height] = icon.sizes.split('x').map(Number);
			expect(pngSize(`static${icon.src}`)).toEqual({ width, height });
		}
		const head = text('src/app.html');
		for (const href of ['/favicon.svg', '/favicon-48.png', '/apple-touch-icon.png', '/site.webmanifest']) expect(head).toContain(`href="${href}"`);
	});
});
