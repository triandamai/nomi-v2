import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Projects against the real backend (App / Projects JyWON): the first project from a starter, its
// workspace opening with Koda on it, the card back on Projects with its files to export, finding and
// filtering, and deleting it (asked first).

const WORKSPACE_URL = /\/projects\/[0-9a-f-]{36}$/;
type Projects = { projects: { id: string; name: string }[] };

test('a first project from a starter, found, exported and deleted', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Projects' }).click();
	await expect(page).toHaveURL(/\/projects$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Projects' })).toBeVisible();

	// The starter goes up into the composer; Start building opens the project's chat.
	await page.getByRole('button', { name: /Menu and pre-order page for a warung/ }).click();
	const composer = page.getByRole('textbox', { name: 'Describe an app to build' });
	await expect(composer).toHaveValue('Menu and pre-order page for a warung');
	await page.getByRole('button', { name: 'Start building' }).click();
	await expect(page).toHaveURL(WORKSPACE_URL);
	await expect(page.getByRole('region', { name: 'Chat' }).getByText('Menu and pre-order page for a warung')).toBeVisible();

	// Back on Projects: its card, found by search, not under Live.
	await page.goto('/projects', { waitUntil: 'networkidle' });
	const card = page.getByRole('article').filter({ hasText: 'New project' });
	await expect(card).toHaveCount(1);
	await page.getByRole('searchbox').fill('nothing like it');
	await expect(page.getByText('No projects match “nothing like it”')).toBeVisible();
	await page.getByRole('button', { name: 'Clear search' }).last().click();
	await expect(card).toHaveCount(1);
	await page.getByRole('radio', { name: /^Live/ }).click();
	await expect(page.getByText('Nothing is live yet')).toBeVisible();
	await page.getByRole('radio', { name: /^All/ }).click();
	await expect(card).toHaveCount(1);

	// Its files, as a zip.
	const id = ((await (await api(account, '/api/projects')).json()) as Projects).projects[0].id;
	const zip = await page.request.get(`/projects/${id}/export`);
	expect([zip.status(), zip.headers()['content-type']]).toEqual([200, 'application/zip']);

	// Deleted from its menu, after asking.
	await card.getByRole('button', { name: /More|Project actions|menu/i }).click();
	await page.getByRole('menuitem', { name: /Delete/ }).click();
	await page.getByRole('dialog', { name: 'Delete New project?' }).getByRole('button', { name: 'Delete project' }).click();
	await expect(page.getByText('New project deleted')).toBeVisible();
	await expect(page.getByRole('button', { name: /Booking page for my barbershop/ })).toBeVisible();
	expect(((await (await api(account, '/api/projects')).json()) as Projects).projects).toEqual([]);
});
