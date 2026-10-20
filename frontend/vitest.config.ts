import { fileURLToPath } from 'node:url';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	resolve: {
		// Helpers under test import the compiled Paraglide messages through SvelteKit's alias.
		alias: { $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) },
	},
	test: {
		projects: [
			{
				extends: true,
				test: {
					name: 'unit',
					include: ['ws-proxy/**/*.test.js', 'src/**/*.test.ts'],
					exclude: ['src/lib/components/**/*.test.ts'],
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
					},
				},
				test: {
					name: 'components',
					include: ['src/lib/components/**/*.test.ts'],
					environment: 'jsdom',
				},
			},
		],
	},
});
