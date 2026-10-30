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
	projects: [
		{
			name: 'chromium',
			// Linux Chromium hints glyph advances to whole pixels, so text runs wider than on the Mac the
			// pen is drawn on and lines wrap early. Unhinted, it measures text as the Mac does.
			use: { ...devices['Desktop Chrome'], launchOptions: { args: ['--font-render-hinting=none'] } },
		},
	],
	webServer: [
		{
			command: 'node e2e/stub-api.mjs',
			url: 'http://127.0.0.1:4180/health',
			reuseExistingServer: !process.env.CI,
		},
		{
			// One build after the other (they share the Vite cache), then the server.
			command: 'npm run build && npm run build:gallery && node server.js',
			url: 'http://localhost:4175',
			reuseExistingServer: !process.env.CI,
			timeout: 180_000,
			// The gallery is on so pen-parity.e2e.ts can hold it to the pen.
			env: { PORT: '4175', HOST: '127.0.0.1', API_URL: 'http://127.0.0.1:4180', ORIGIN: 'http://localhost:4175', PUBLIC_DS_GALLERY: '1' },
		},
		{
			// The static gallery, served under a nested path (gallery-static.e2e.ts).
			command: 'node e2e/static-gallery-server.mjs',
			url: 'http://127.0.0.1:4181/health',
			reuseExistingServer: !process.env.CI,
			timeout: 240_000,
		},
	],
});
