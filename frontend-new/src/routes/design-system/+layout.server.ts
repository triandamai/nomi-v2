import { error } from '@sveltejs/kit';
import { dev } from '$app/environment';
import { env } from '$env/dynamic/public';
import { galleryEnabled } from '$lib/gallery/enabled';

// A tool for building and reviewing the design system, not part of the app: on by default in
// development, and wherever PUBLIC_DS_GALLERY=1 (CI, review deployments). `npm run build:gallery`
// prerenders it into static files to share, so there it's always on.
export const prerender = __DS_STATIC__;

export function load() {
	if (__DS_STATIC__) return;
	if (!galleryEnabled(env.PUBLIC_DS_GALLERY ?? (dev ? '1' : undefined))) error(404, 'Not found');
}
