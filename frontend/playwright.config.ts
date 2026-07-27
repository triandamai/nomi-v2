import { defineConfig } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const backendDir = path.resolve(__dirname, '../backend');

export default defineConfig({
	webServer: [
		{
			command: 'cargo run',
			cwd: backendDir,
			port: 8080,
			timeout: 120_000,
			reuseExistingServer: !process.env.CI,
			env: {
				DATABASE_URL: process.env.DATABASE_URL ?? 'postgres://postgres:postgres@localhost:5432/nomi_dev',
				JWT_SECRET: 'e2e-test-secret-do-not-use-in-prod',
				LLM_PROVIDER: 'fake',
				EMBEDDING_PROVIDER: 'fake',
			},
		},
		{
			command: 'npm run build && npm run preview',
			port: 4173,
			reuseExistingServer: !process.env.CI,
		},
	],
	use: {
		baseURL: 'http://localhost:4173',
	},
	testMatch: '**/*.e2e.{ts,js}',
	testDir: 'e2e',
});
