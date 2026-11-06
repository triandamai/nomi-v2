import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	resolve: {
		// Helpers under test import the compiled Paraglide messages through SvelteKit's alias.
		// SvelteKit's virtual `$env` modules don't exist outside its Vite plugin either.
		alias: {
			$lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
			'$env/dynamic/private': fileURLToPath(new URL('./src/test-stubs/env-dynamic-private.ts', import.meta.url)),
			'$env/dynamic/public': fileURLToPath(new URL('./src/test-stubs/env-dynamic-public.ts', import.meta.url)),
		},
	},
	test: {
		// `npm run test:coverage` runs both projects into one report: `text-summary` for the
		// terminal, `json-summary` for CI's job summary, `html` to browse locally.
		coverage: {
			provider: 'v8',
			include: ['src/**/*.{ts,svelte}', 'ws-proxy/**/*.js'],
			exclude: ['src/lib/paraglide/**', 'src/test-stubs/**', '**/*.test.ts', '**/*.test.js', '**/*.d.ts'],
			reporter: ['text-summary', 'json-summary', 'html'],
			reportsDirectory: './coverage',
			// Floors at the coverage reached (Nov 2026, from 22.8% lines); raise them as tests grow.
			thresholds: { lines: 55, statements: 54, functions: 54, branches: 44 },
		},
		projects: [
			{
				extends: true,
				test: {
					name: 'unit',
					include: ['ws-proxy/**/*.test.js', 'src/**/*.test.ts'],
					exclude: ['src/lib/components/**/*.test.ts', 'src/**/*.svelte.test.ts'],
					environment: 'node',
				},
			},
			{
				// Components mount into jsdom with Svelte's own `mount`. Only these tests resolve
				// packages' browser builds: `ws`, which the ws-proxy tests use, throws in its own.
				extends: true,
				plugins: [svelte()],
				resolve: {
					conditions: ['browser'],
					// SvelteKit's virtual modules don't exist outside its Vite plugin; drawer components need one.
					alias: {
						'$app/state': fileURLToPath(new URL('./src/test-stubs/app-state.ts', import.meta.url)),
						'$app/environment': fileURLToPath(new URL('./src/test-stubs/app-environment.ts', import.meta.url)),
						'$app/forms': fileURLToPath(new URL('./src/test-stubs/app-forms.ts', import.meta.url)),
						'$app/navigation': fileURLToPath(new URL('./src/test-stubs/app-navigation.ts', import.meta.url)),
					},
				},
				test: {
					name: 'components',
					// Components, and route pages and rune modules (`*.svelte.test.ts`) mounted the same way.
					include: ['src/lib/components/**/*.test.ts', 'src/**/*.svelte.test.ts'],
					environment: 'jsdom',
					setupFiles: ['./src/test-stubs/jsdom-setup.ts'],
				},
			},
		],
	},
});
