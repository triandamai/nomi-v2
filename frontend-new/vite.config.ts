import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { playwright } from '@vitest/browser-playwright';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [
		tailwindcss(),
		// English and Indonesian. The person's saved language lives in the cookie; before sign-in,
		// the browser's language decides.
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['cookie', 'preferredLanguage', 'baseLocale'],
		}),
		sveltekit({
			compilerOptions: {
				// Runes everywhere except in libraries. Can be removed in Svelte 6.
				runes: ({ filename }) => (filename.split(/[/\\]/).includes('node_modules') ? undefined : true),
			},
			adapter: adapter(),
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
					include: ['src/**/*.test.ts'],
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
