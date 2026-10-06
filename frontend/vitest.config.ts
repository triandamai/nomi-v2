import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	resolve: {
		// Helpers under test import the compiled Paraglide messages through SvelteKit's alias.
		alias: { $lib: fileURLToPath(new URL('./src/lib', import.meta.url)) },
	},
	test: {
		include: ['ws-proxy/**/*.test.js', 'src/**/*.test.ts'],
		environment: 'node'
	}
});
