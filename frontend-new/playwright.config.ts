import { defineConfig, devices } from '@playwright/test';

// End-to-end tests run the production server (server.js) on 4175 (the old frontend's e2e keeps
// 4173). The smoke tests talk to a stub backend; flow tests will add the real one.
export default defineConfig({
	testDir: 'e2e',
	testMatch: '**/*.e2e.ts',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	reporter: process.env.CI ? 'github' : 'list',
	use: { baseURL: 'http://localhost:4175' },
	projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
	webServer: [
		{
			command: 'node e2e/stub-api.mjs',
			url: 'http://127.0.0.1:4180/health',
			reuseExistingServer: !process.env.CI,
		},
		{
			command: 'npm run build && node server.js',
			url: 'http://localhost:4175',
			reuseExistingServer: !process.env.CI,
			timeout: 180_000,
			env: { PORT: '4175', HOST: '127.0.0.1', API_URL: 'http://127.0.0.1:4180', ORIGIN: 'http://localhost:4175' },
		},
	],
});
