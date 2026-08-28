import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		include: ['ws-proxy/**/*.test.js', 'src/**/*.test.ts'],
		environment: 'node'
	}
});
