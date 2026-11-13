// Where the real backend and its mail catcher are, and how the backend is started for e2e.

export const BACKEND_URL = process.env.E2E_BACKEND_URL ?? 'http://127.0.0.1:8080';
export const MAILPIT_URL = process.env.MAILPIT_URL ?? 'http://localhost:8025';
/** Where this run's production server listens. Two runs side by side (a backend is shared on
 * 8080) each take their own: `E2E_APP_PORT=4177 npm run test:e2e:backend`. */
export const APP_PORT = process.env.E2E_APP_PORT ?? '4176';
export const APP_URL = `http://localhost:${APP_PORT}`;

/** The backend's environment for e2e: its fake model, codes on, every email to Mailpit. */
export function backendEnv(): Record<string, string> {
	return {
		DATABASE_URL: process.env.E2E_DATABASE_URL ?? 'postgres://nomi:nomi@localhost:5432/nomi_e2e_v2',
		// Test-only placeholders, never used outside an e2e database.
		JWT_SECRET: 'e2e-secret-do-not-use-in-prod',
		SETTINGS_ENCRYPTION_KEY: '0'.repeat(64),
		LLM_PROVIDER: 'fake',
		EMBEDDING_PROVIDER: 'fake',
		AUTH_EMAIL_CODES: 'on',
		SMTP_HOST: process.env.E2E_SMTP_HOST ?? '127.0.0.1',
		SMTP_PORT: process.env.E2E_SMTP_PORT ?? '1025',
		SMTP_SECURITY: 'none',
		MAIL_FROM: 'Nomi <no-reply@nomi.test>',
		APP_URL,
		MQTT_BROKER_HOST: process.env.E2E_MQTT_HOST ?? '127.0.0.1',
		MQTT_BROKER_PORT: process.env.E2E_MQTT_PORT ?? '1883',
		RUST_LOG: process.env.RUST_LOG ?? 'warn',
	};
}
