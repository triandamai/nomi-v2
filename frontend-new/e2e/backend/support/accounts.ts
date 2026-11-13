import type { BrowserContext } from '@playwright/test';
import { APP_URL, BACKEND_URL } from './env';
import { codeFor } from './mailpit';

export type Account = { email: string; password: string; workspace: string; tokens: { access_token: string; refresh_token: string } };

let made = 0;

/** A fresh address for one test, so tests never share an account. */
export function newEmail(label = 'e2e'): string {
	return `${label}-${Date.now()}-${process.pid}-${made++}@nomi.test`;
}

async function post(path: string, body: unknown, accessToken?: string): Promise<Response> {
	return fetch(`${BACKEND_URL}${path}`, {
		method: 'POST',
		headers: { 'content-type': 'application/json', ...(accessToken && { authorization: `Bearer ${accessToken}` }) },
		body: JSON.stringify(body),
	});
}

/**
 * A confirmed account made through the API as a person would make it: register, then the code
 * from the confirmation email. For tests that start signed in.
 */
export async function createAccount(options: { email?: string; password?: string; workspace?: string; language?: 'en' | 'id' } = {}): Promise<Account> {
	const email = options.email ?? newEmail();
	const password = options.password ?? 'correct-horse-9';
	const workspace = options.workspace ?? 'E2E space';
	const started = Date.now();
	const registered = await post('/api/auth/register', { email, password, org: { mode: 'create', name: workspace }, language: options.language ?? 'en' });
	if (!registered.ok) throw new Error(`register ${email}: ${registered.status} ${await registered.text()}`);
	const { verification } = (await registered.json()) as { verification: { challenge_id: string } };
	const code = await codeFor(email, /confirm/i, { after: started - 1000 });
	const verified = await post('/api/auth/verify', { challenge_id: verification.challenge_id, code });
	if (!verified.ok) throw new Error(`verify ${email}: ${verified.status} ${await verified.text()}`);
	return { email, password, workspace, tokens: (await verified.json()) as Account['tokens'] };
}

/** Signs the browser in as `account`, the way the app keeps a sign-in (httpOnly cookies). */
export async function signInAs(context: BrowserContext, account: Account, baseURL = APP_URL): Promise<void> {
	await context.addCookies([
		{ name: 'access_token', value: account.tokens.access_token, url: baseURL, httpOnly: true, sameSite: 'Lax' },
		{ name: 'refresh_token', value: account.tokens.refresh_token, url: baseURL, httpOnly: true, sameSite: 'Lax' },
	]);
}

/** Calls the backend as `account`. */
export function api(account: Account, path: string, init: RequestInit = {}): Promise<Response> {
	return fetch(`${BACKEND_URL}${path}`, {
		...init,
		headers: { 'content-type': 'application/json', authorization: `Bearer ${account.tokens.access_token}`, ...(init.headers ?? {}) },
	});
}
