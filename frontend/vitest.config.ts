import { defineConfig } from 'vitest/config';

export default defineConfig({
	test: {
		include: ['ws-proxy/**/*.test.js'],
		environment: 'node',
		testTimeout: 10000
	}
});
