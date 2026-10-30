import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import adapterStatic from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vitest/config';
import { devWsProxy } from './vite-plugins/dev-ws-proxy.ts';
import { appVersion } from './app-version.js';

const version = appVersion();
// `npm run build:gallery` (DS_STATIC=1): the design-system gallery alone, prerendered into static
// files in build-gallery/ that work from any path on any host, to share with the team.
const galleryStatic = process.env.DS_STATIC === '1';

export default defineConfig({
	// Shown in the app (see $lib/version) and sent to the backend with every call.
	define: { __APP_VERSION__: JSON.stringify(version), __DS_STATIC__: JSON.stringify(galleryStatic) },
	// Rolldown's plugin-timing report prints as a warning on every build; it's a profiling aid, not
	// a problem with the build.
	build: { rolldownOptions: { checks: { pluginTimings: false } } },
	plugins: [
		tailwindcss(),
		// English and Indonesian. The person's saved language lives in the cookie; before sign-in,
		// the browser's language decides.
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['cookie', 'preferredLanguage', 'baseLocale'],
		}),
		// The live-update relay (ws-proxy), the same one server.js runs in production.
		devWsProxy({ clientVersion: version }),
		sveltekit({
			compilerOptions: {
				// Runes everywhere except in libraries. Can be removed in Svelte 6.
				runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true),
			},
			adapter: galleryStatic ? adapterStatic({ pages: 'build-gallery', assets: 'build-gallery', strict: false }) : adapter(),
			// The static gallery builds into its own folder, so it never clobbers the app's build.
			outDir: galleryStatic ? '.svelte-kit-gallery' : '.svelte-kit',
			prerender: galleryStatic ? { crawl: false, entries: ['/design-system/nomi', '/design-system/chat', '/design-system/build'], handleUnseenRoutes: 'ignore' } : undefined,
			// SvelteKit's own update check follows the same version: after a deploy, the next
			// navigation loads the new build.
			version: { name: version, pollInterval: 5 * 60_000 },
		}),
	],
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				// Pure TypeScript and server code: no DOM.
				extends: './vite.config.ts',
				test: {
					name: 'unit',
					environment: 'node',
					include: ['src/**/*.test.ts', 'ws-proxy/**/*.test.js'],
					exclude: ['src/**/*.svelte.test.ts'],
				},
			},
			{
				// Components, rendered in a real Chromium with the app's CSS, so layout, focus, the
				// top layer and design tokens behave as they do for the person.
				extends: './vite.config.ts',
				test: {
					name: 'components',
					include: ['src/**/*.svelte.test.ts'],
					setupFiles: ['./src/lib/ds/test-setup.ts'],
					browser: {
						enabled: true,
						headless: true,
						provider: playwright(),
						instances: [{ browser: 'chromium' }],
					},
				},
			},
		],
	},
});
