import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Settings › Workspaces against the real backend (App / Settings / Workspaces l222Mz): the
// workspace made at sign-up, a new one created, and projects set to come from it, from its card
// and from Defaults.

type Orgs = { orgs: { id: string; name: string }[]; project_org_id: string | null };

test('workspaces: create one and choose where projects come from', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/connections', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Workspaces' }).click();
	await expect(page).toHaveURL(/\/settings\/workspaces$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Workspaces' })).toBeVisible();
	await expect(page.getByRole('banner').getByText('1 workspace')).toBeVisible();
	const first = page.getByRole('article', { name: account.workspace });
	await expect(first).toBeVisible();
	await expect(first.getByText('Owner · only you')).toBeVisible();

	// A name is needed; then it's made.
	await page.getByRole('button', { name: /Create a workspace/ }).click();
	const dialog = page.getByRole('dialog', { name: 'Create a workspace' });
	await dialog.getByRole('button', { name: 'Create' }).click();
	await expect(dialog.getByText('Give the workspace a name.')).toBeVisible();
	await dialog.getByLabel('Name').fill('Kopi Kita');
	await dialog.getByRole('button', { name: 'Create' }).click();
	await expect(page.getByText('Created Kopi Kita')).toBeVisible();
	const kopi = page.getByRole('article', { name: 'Kopi Kita' });
	await expect(kopi).toBeVisible();
	await expect(page.getByRole('banner').getByText('2 workspaces')).toBeVisible();

	// Projects come from Kopi Kita now.
	await kopi.getByRole('button', { name: 'Use for projects' }).click();
	await expect(page.getByText('Projects now come from Kopi Kita')).toBeVisible();
	await expect(kopi.getByText('Projects come from here')).toBeVisible();
	const orgs = async () => (await (await api(account, '/api/orgs')).json()) as Orgs;
	const kopiId = (await orgs()).orgs.find((org) => org.name === 'Kopi Kita')!.id;
	expect((await orgs()).project_org_id).toBe(kopiId);
	await expect(kopi.getByRole('link', { name: 'Settings' })).toHaveAttribute('href', `/settings/workspaces/${kopiId}`);

	// And back, from Defaults.
	const firstId = (await orgs()).orgs.find((org) => org.name === account.workspace)!.id;
	await page.getByRole('combobox', { name: 'New projects go to' }).selectOption(firstId);
	await expect(first.getByText('Projects come from here')).toBeVisible();
	await expect.poll(async () => (await orgs()).project_org_id).toBe(firstId);
});
