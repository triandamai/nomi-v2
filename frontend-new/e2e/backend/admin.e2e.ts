import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAgain, signInAs, type Account } from './support/accounts';
import { makePlatformAdmin } from './support/db';

// The v2 admin console at /admin against the real backend: who may open it (signed out, no admin
// area, a platform admin), its navigation, the overview, the crew at work live, and usage by
// month. A platform admin is made in the database (nothing in the API makes one) and signs in again
// so the token carries it.

/** A platform admin, signed in with a token that says so. */
async function platformAdmin(): Promise<Account> {
	const account = await createAccount({ workspace: 'Admin HQ' });
	await makePlatformAdmin(account.email);
	return signInAgain(account);
}

async function open(page: Page, path: string) {
	await page.goto(path, { waitUntil: 'networkidle' });
}

const nav = (page: Page) => page.getByRole('navigation', { name: 'Admin' });

test('someone signed out signs in first, and comes back to the console', async ({ request }) => {
	const response = await request.get('/admin/usage', { maxRedirects: 0 });
	expect(response.status()).toBe(303);
	expect(response.headers().location).toBe('/sign-in?redirect_to=%2Fadmin%2Fusage');
});

test('someone without an admin area is told so, and nothing of the console shows', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const response = await page.goto('/admin');
	expect(response?.status()).toBe(403);
	await expect(page.getByText('No access to the admin console')).toBeVisible();
	await expect(nav(page)).toHaveCount(0);
	// The backend says the same, whatever the console shows.
	expect((await api(account, '/api/admin/dashboard')).status).toBe(403);
});

test('a platform admin sees every area, and the overview of how Nomi is doing', async ({ page, context }) => {
	const admin = await platformAdmin();
	await signInAs(context, admin);
	await open(page, '/admin');

	for (const area of ['Overview', 'Live agents', 'Usage', 'Users & access', 'Plans', 'Notifications', 'Models', 'Providers', 'Embedding', 'Tools', 'Custom agents', 'Support', 'Maintenance', 'Integrations', 'Configuration']) {
		await expect(nav(page).getByRole('link', { name: area, exact: true })).toBeVisible();
	}
	await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
	await expect(page.getByText('At a glance', { exact: true })).toBeVisible();
	await expect(page.getByText('The overview couldn’t load')).toHaveCount(0);
});

test('the crew at work shows live, and says when nobody is', async ({ page, context }) => {
	const admin = await platformAdmin();
	await signInAs(context, admin);
	await open(page, '/admin/agents');

	await expect(page.getByRole('heading', { level: 1, name: 'Live agents' })).toBeVisible();
	await expect(page.getByText('Live', { exact: true })).toBeVisible();
	await expect(page.getByText('The crew’s work couldn’t load')).toHaveCount(0);
});

test('usage shows a month and moves between months', async ({ page, context }) => {
	const admin = await platformAdmin();
	// Something to count this month: a chat the fake model answers.
	const { session_id } = (await (await api(admin, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	expect((await api(admin, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Halo' }) })).ok).toBe(true);
	await signInAs(context, admin);
	await open(page, '/admin/usage');

	await expect(page.getByRole('heading', { level: 1, name: 'Usage' })).toBeVisible();
	await expect(page.getByText('Usage couldn’t load')).toHaveCount(0);
	await page.getByRole('link', { name: 'Previous month' }).click();
	await expect(page).toHaveURL(/\/admin\/usage\?month=\d{4}-\d{2}$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Usage' })).toBeVisible();
});
