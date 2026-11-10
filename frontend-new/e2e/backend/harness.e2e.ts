import { expect, test } from '@playwright/test';
import { api, createAccount, newEmail, signInAs } from './support/accounts';
import { BACKEND_URL } from './support/env';
import { codeFor, emailsTo } from './support/mailpit';

// The harness itself, against the real backend: accounts are made as people make them (the
// code arrives by email), and a signed-in browser is that person on the backend and its broker.

test('a new account confirms its email with the code the backend emailed it', async () => {
	const account = await createAccount();
	const me = (await (await api(account, '/api/whoami')).json()) as { sub: string; permissions: string[] };
	expect(me.sub).toMatch(/^[0-9a-f-]{36}$/);
	expect((await emailsTo(account.email)).map((message) => message.Subject)).toEqual([expect.stringMatching(/^Confirm your email for Nomi: \d{6}$/)]);
});

test('signing in again with the password sends a fresh code', async () => {
	const account = await createAccount({ email: newEmail('again') });
	const started = Date.now();
	const response = await fetch(`${BACKEND_URL}/api/auth/login`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ email: account.email, password: account.password }),
	});
	expect(((await response.json()) as { verification: { purpose: string } }).verification.purpose).toBe('login');
	expect(await codeFor(account.email, /sign-in code/i, { after: started - 1000 })).toMatch(/^\d{6}$/);
});

test('a signed-in browser gets the real live-update feed through the relay', async ({ page, context }) => {
	await signInAs(context, await createAccount());
	await page.goto('/');
	const outcome = await page.evaluate(
		() =>
			new Promise<{ opened: boolean; code?: number }>((resolve) => {
				const socket = new WebSocket(`${location.origin.replace('http', 'ws')}/activity/ws`);
				let opened = false;
				socket.onopen = () => {
					opened = true;
					// The feed stays open while the person is here; a refused one closes with 4401.
					setTimeout(() => {
						socket.close();
						resolve({ opened });
					}, 1500);
				};
				socket.onclose = (event) => resolve({ opened, code: event.code });
			}),
	);
	expect(outcome).toEqual({ opened: true });
});
