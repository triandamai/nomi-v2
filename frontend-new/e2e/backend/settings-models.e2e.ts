import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedNomiModel } from './support/seed';

// Settings › Models & providers against the real backend (App / Settings / Models & providers
// RbTgo, Connect voUxB): the defaults and Nomi's own models, the Connect dialog's list and search,
// and a provider the backend can't reach turned away with its reason, nothing kept.

test('models and providers, and connecting one that can’t be reached', async ({ page, context }) => {
	const account = await createAccount();
	await seedNomiModel(account.email);
	await signInAs(context, account);
	await page.goto('/settings/plan', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Models & providers' }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);
	await expect(page.getByRole('banner').getByText(/providers? · \d+ models? in picker/)).toBeVisible();
	await expect(page.getByRole('heading', { level: 2, name: 'Defaults' })).toBeVisible();
	await expect(page.getByRole('combobox', { name: 'New chats start with' })).toBeVisible();
	await expect(page.getByRole('button', { name: /Included/ })).toHaveAttribute('aria-expanded', 'true');

	// The dialog, from the top bar: the list, a search, a provider.
	await page.getByRole('banner').getByRole('link', { name: 'Connect a provider' }).click();
	await expect(page).toHaveURL(/\?connect$/);
	const dialog = page.getByRole('dialog', { name: 'Pick a provider' });
	await expect(dialog).toBeVisible();
	await dialog.getByRole('textbox', { name: 'Search providers' }).fill('anthro');
	await dialog.getByRole('button', { name: 'Anthropic' }).click();
	await expect(page).toHaveURL(/\?connect=anthropic$/);
	await expect(page.getByRole('dialog', { name: 'Connect Anthropic' }).getByLabel('API key')).toHaveAttribute('type', 'password');

	// A custom endpoint nobody answers at: turned away with the reason, and nothing kept.
	await page.getByRole('dialog').getByRole('button', { name: /Custom endpoint/ }).click();
	const pane = page.getByRole('dialog', { name: /^Connect / });
	await pane.getByLabel('Address').fill('https://127.0.0.1:9/v1');
	await pane.getByRole('button', { name: 'Check and connect' }).click();
	await expect(pane.getByRole('alert')).toBeVisible({ timeout: 30_000 });
	await expect(pane.getByLabel('Address')).toHaveAttribute('aria-invalid', 'true');
	await pane.getByRole('button', { name: 'Cancel' }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);
	const { connections } = (await (await api(account, '/api/llm/connections')).json()) as { connections: unknown[] };
	expect(connections).toEqual([]);
});
