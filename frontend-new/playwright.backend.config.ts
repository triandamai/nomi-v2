import { defineConfig, devices } from '@playwright/test';
import { BACKEND_URL, MAILPIT_URL, backendEnv } from './e2e/backend/support/env';

// The flows end to end against the real backend (backend/, with its fake model so replies are
// known) and its real dependencies: Postgres, the EMQX broker, and Mailpit catching the emails
// that carry sign-in codes. The production server (server.js) runs on 4176.
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
	use: { baseURL: 'http://localhost:4176', trace: 'retain-on-failure' },
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
			url: 'http://localhost:4176/health',
			reuseExistingServer: !process.env.CI,
			timeout: 240_000,
			env: { PORT: '4176', HOST: '127.0.0.1', API_URL: BACKEND_URL, ORIGIN: 'http://localhost:4176' },
		},
	],
});
