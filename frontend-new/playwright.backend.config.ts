import { defineConfig, devices } from '@playwright/test';
import { APP_PORT, APP_URL, BACKEND_URL, MAILPIT_URL, backendEnv } from './e2e/backend/support/env';

// The flows end to end against the real backend (backend/, with its fake model so replies are
// known) and its real dependencies: Postgres, the EMQX broker, and Mailpit catching the emails
// that carry sign-in codes. The production server (server.js) runs on 4176 (E2E_APP_PORT).
//
// Locally: the backend's docker compose (Postgres + EMQX), Mailpit
// (`docker run -d -p 1025:1025 -p 8025:8025 axllent/mailpit`), an e2e database
// (`createdb nomi_e2e_v2`, or set E2E_DATABASE_URL), then `npm run test:e2e:backend`.
export default defineConfig({
	testDir: 'e2e/backend',
	testMatch: '**/*.e2e.ts',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	reporter: process.env.CI ? 'github' : 'list',
	timeout: 60_000,
	// A real backend under a parallel run (it emails a code on every password sign-in) can take
	// longer than Playwright's 5 s to move a page on.
	expect: { timeout: 15_000 },
	use: { baseURL: APP_URL, trace: 'retain-on-failure' },
	projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'], launchOptions: { args: ['--font-render-hinting=none'] } } }],
	webServer: [
		{
			// Built once by CI (or `cargo build --bin nomi-orchestrator`); cargo only runs it.
			command: 'cargo run --quiet --bin nomi-orchestrator',
			cwd: '../backend',
			url: `${BACKEND_URL}/api/maintenance`,
			reuseExistingServer: !process.env.CI,
			timeout: 600_000,
			env: backendEnv(),
		},
		{
			command: `node -e "fetch('${MAILPIT_URL}/api/v1/info').then((r) => process.exit(r.ok ? 0 : 1), () => process.exit(1))" && npm run build && node server.js`,
			// /health: `/` sends a signed-out visitor to sign in.
			url: `${APP_URL}/health`,
			reuseExistingServer: !process.env.CI,
			timeout: 240_000,
			env: { PORT: APP_PORT, HOST: '127.0.0.1', API_URL: BACKEND_URL, ORIGIN: APP_URL },
		},
	],
});
