import { expect, test } from '@playwright/test';
import { createAccount, signInAs } from './support/accounts';

// Settings › Connections against the real backend (App / Settings / Connections OBmgS): nothing
// connected yet, the apps to add, and a Hevy key and a GitHub token turned away with the reason,
// nothing kept. Google needs a client this backend isn't given, so it says it isn't set up.

test('connections: what can be added, and a key or token that’s refused', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/memory', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Connections' }).click();
	await expect(page).toHaveURL(/\/settings\/connections$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Connections' })).toBeVisible();
	await expect(page.getByRole('banner').getByText('0 connected')).toBeVisible();
	await expect(page.getByText('Nothing is connected yet.', { exact: false })).toBeVisible();
	for (const name of ['Google Workspace', 'GitHub', 'Hevy']) await expect(page.getByRole('article', { name })).toBeVisible();

	// Hevy: the key is checked with Hevy as it connects; a made-up one comes back with the reason.
	await page.getByRole('button', { name: 'Add Hevy' }).click();
	const hevy = page.getByRole('dialog', { name: 'Connect Hevy' });
	await expect(hevy.getByLabel('API key')).toHaveAttribute('type', 'password');
	await hevy.getByLabel('API key').fill('not-a-real-key');
	await hevy.getByRole('button', { name: 'Check and connect' }).click();
	await expect(hevy.getByText(/^Hevy (didn’t accept that key|couldn’t be reached)/)).toBeVisible({ timeout: 20_000 });
	await hevy.getByRole('button', { name: 'Cancel' }).click();
	await expect(hevy).toBeHidden();

	// GitHub: a token GitHub doesn't know is turned away too.
	await page.getByRole('button', { name: 'Add GitHub' }).click();
	const github = page.getByRole('dialog', { name: 'Connect GitHub' });
	await github.getByLabel('Personal access token').fill('ghp_notarealtoken');
	await github.getByRole('button', { name: 'Connect with token' }).click();
	await expect(github.getByText(/^GitHub (didn't accept the token|couldn't be reached)/)).toBeVisible({ timeout: 30_000 });
	await page.keyboard.press('Escape');

	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('banner').getByText('0 connected')).toBeVisible();
});
