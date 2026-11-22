import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import adapterStatic from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vitest/config';
import { devWsProxy } from './vite-plugins/dev-ws-proxy.ts';
import { docsOnlyRoutes } from './vite-plugins/docs-only-routes.ts';
import { dsApi } from './src/lib/docs/api/plugin.ts';
import { appVersion } from './app-version.js';
import { releaseNotes } from './release-notes.js';

const version = appVersion();
// `npm run build:gallery` (DS_STATIC=1): the design-system gallery alone, prerendered into static
// files in build-gallery/ that work from any path on any host, to share with the team.
const galleryStatic = process.env.DS_STATIC === '1';

export default defineConfig({
	// Shown in the app (see $lib/version) and sent to the backend with every call.
	// The release notes CI read from GitHub for this build's environment (release-notes.js): the
	// app shows them without calling GitHub.
	define: { __APP_VERSION__: JSON.stringify(version), __DS_STATIC__: JSON.stringify(galleryStatic), __RELEASE_NOTES__: JSON.stringify(releaseNotes()) },
	// Rolldown's plugin-timing report prints as a warning on every build; it's a profiling aid, not
	// a problem with the build.
	// The static copy is served from any path: an image imported in code is found next to the code
	// that asked for it, not at the site's root.
	experimental: galleryStatic ? { renderBuiltUrl: (_file: string, { hostType }: { hostType: string }) => (hostType === 'js' ? { relative: true } : undefined) } : undefined,
	build: {
		rolldownOptions: {
			checks: { pluginTimings: false },
		},
	},
	plugins: [
		{
			name: 'open-pencil-worker-fix',
			enforce: 'pre',
			transform(code, id) {
				if (id.includes('@open-pencil/core') && (code.includes('export-worker.ts') || code.includes('worker.ts'))) {
					return {
						code: code
							.replace(/new Worker\(new URL\(['"]\.\/export-worker\.ts['"],\s*import\.meta\.url\)[^)]*\)/g, 'null')
							.replace(/new Worker\(new URL\(['"]\.\/worker\.ts['"],\s*import\.meta\.url\)[^)]*\)/g, 'null'),
						map: null
					};
				}
			}
		},
		...(galleryStatic ? [docsOnlyRoutes()] : []),
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
		// virtual:ds-api: every component's props and relations, read from source for the docs.
		dsApi(),
		sveltekit({
			compilerOptions: {
				// Runes everywhere except in libraries. Can be removed in Svelte 6.
				runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true),
			},
			adapter: galleryStatic ? adapterStatic({ pages: 'build-gallery', assets: 'build-gallery', strict: false }) : adapter(),
			// The static gallery builds into its own folder, so it never clobbers the app's build.
			outDir: galleryStatic ? '.svelte-kit-gallery' : '.svelte-kit',
			// Share hosts can reserve names starting with `_`, so the static gallery keeps its assets in app/.
			appDir: galleryStatic ? 'app' : '_app',
			// Every design-system page: the docs, the sheets, and (through their `entries`) each
			// component and pattern. Links there end in .html, so nothing is found by crawling.
			prerender: galleryStatic ? { crawl: false, entries: ['*'], handleUnseenRoutes: 'ignore' } : undefined,
			// The static copy goes to share hosts that cap a site at about 500 files, and every route of
			// the app is built into it (though only the design system's are served): one bundle for
			// all of them keeps a new page from costing a file there.
			output: galleryStatic ? { bundleStrategy: 'single' } : undefined,
			// SvelteKit's own update check follows the same version: after a deploy, the next
			// navigation loads the new build.
			version: { name: version, pollInterval: 5 * 60_000 },
		}),
	],
	test: {
		expect: { requireAssertions: true },
		// `npm run test:coverage` runs both projects and merges them into one report: `text` for the
		// terminal, `json-summary` for CI's job summary, `html` to browse locally.
		coverage: {
			provider: 'v8',
			include: ['src/**/*.{ts,svelte}', 'ws-proxy/**/*.js', 'server.js'],
			exclude: ['src/lib/paraglide/**', '**/*.test.ts', '**/*.test.js', '**/*.d.ts'],
			reporter: ['text-summary', 'json-summary', 'html'],
			reportsDirectory: './coverage',
		},
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
