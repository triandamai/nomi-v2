import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Sharing a chat against the real backend (Flow / Chat — Share & delete TnscX, Shared link b8Flw8):
// the share dialog makes a link, a person with no account opens it read-only, a link for signed-in
// people asks them to sign in, and a link turned off isn't available.

const REPLY = 'This is a fake response for local development and testing.';

test('a chat is shared, opened read-only by someone signed out, limited to signed-in people, then turned off', async ({ page, context, browser }) => {
	const account = await createAccount();
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Rencana makan minggu ini' }) });
	await signInAs(context, account);
	await page.goto(`/chat/${session_id}`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('main').getByText(REPLY)).toBeVisible();

	await page.getByRole('banner').getByRole('button', { name: /^Share/ }).click();
	const dialog = page.getByRole('dialog', { name: 'Share' });
	await expect(dialog).toBeVisible();
	const url = (await dialog.getByText(/\/shared\//).textContent())!.trim();
	const link = url.startsWith('http') ? url : `http://${url}`;

	// Someone with no account opens it.
	const visitor = await browser.newContext();
	const viewer = await visitor.newPage();
	await viewer.goto(link);
	await expect(viewer.getByText('Read-only', { exact: true })).toBeVisible();
	await expect(viewer.getByText('Rencana makan minggu ini')).toBeVisible();
	await expect(viewer.getByRole('article').getByText(REPLY)).toBeVisible();
	await expect(viewer.getByText('amounts and names hidden', { exact: false })).toBeVisible();

	// Only people signed in may open it now.
	await dialog.getByRole('switch', { name: 'Anyone with the link can view' }).click();
	await expect.poll(async () => (await viewer.request.get(link)).status()).toBe(200);
	await viewer.reload();
	await expect(viewer.getByRole('heading', { name: 'Sign in to open this link' })).toBeVisible();

	// Turned off: it isn't available to anyone.
	const { shares } = (await (await api(account, '/api/shares')).json()) as { shares: { id: string; session_id: string }[] };
	const share = shares.find((one) => one.session_id === session_id)!;
	expect((await api(account, `/api/sessions/${session_id}/shares/${share.id}`, { method: 'DELETE' })).ok).toBe(true);
	await viewer.reload();
	await expect(viewer.getByRole('heading', { name: 'This link isn’t available' })).toBeVisible();
	await visitor.close();
});
