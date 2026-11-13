import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Settings › Profile against the real backend (App / Settings / Profile U9Gnrg): the settings
// frame, the numbers of a new account, saving the public profile, the username's rules and
// whether it's free, discarding, the photo, and Esc back to the app.

const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==', 'base64');
const unique = () => `e2e_${Date.now().toString(36)}${Math.floor(Math.random() * 1000)}`;

test('the profile is opened from Settings, edited, saved and kept', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await expect(page).toHaveURL(/\/settings\/profile$/);
	await expect(page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Profile' })).toBeVisible();
	await expect(page.getByRole('banner').getByText('Profile', { exact: true })).toBeVisible();

	// A new account's numbers.
	await expect(page.getByRole('term').filter({ hasText: 'Current streak' })).toBeVisible();
	await expect(page.getByText('nothing built yet')).toBeVisible();
	await expect(page.getByRole('img', { name: /tokens in the last year/ })).toBeVisible();
	await expect(page.getByText('Publish a project to feature it here.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Add a live project' })).toBeDisabled();

	// Name and username: two changes, saved, and still there after a reload.
	const username = unique();
	await page.getByRole('textbox', { name: 'Display name' }).fill('Sari Wulandari');
	await page.getByRole('textbox', { name: 'Username' }).fill(username);
	await expect(page.getByText('2 unsaved changes')).toBeVisible();
	await expect(page.getByText(`/@${username} · available`)).toBeVisible();
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByText('Profile saved')).toBeVisible();
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: 'Sari Wulandari' })).toBeVisible();
	await expect(page.getByText(`@${username} · joined`)).toBeVisible();
	const saved = (await (await api(account, '/api/profile')).json()) as { display_name: string; username: string };
	expect([saved.display_name, saved.username]).toEqual(['Sari Wulandari', username]);

	// Discard puts a change back.
	await page.getByRole('switch', { name: 'Show crew on profile' }).click();
	await expect(page.getByText('1 unsaved change')).toBeVisible();
	await page.getByRole('button', { name: 'Discard' }).click();
	await expect(page.getByRole('switch', { name: 'Show crew on profile' })).toHaveAttribute('aria-checked', 'false');
	await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();

	// Esc goes back to the app.
	await page.locator('body').click({ position: { x: 600, y: 10 } });
	await page.keyboard.press('Escape');
	await expect(page).toHaveURL(/\/$/);
});

test('a username is checked as it’s typed: its rules here, whether it’s free there', async ({ page, context }) => {
	const [other, account] = [await createAccount(), await createAccount()];
	const taken = unique();
	await api(other, '/api/profile', { method: 'PUT', body: JSON.stringify({ username: taken }) });
	await signInAs(context, account);
	await page.goto('/settings/profile', { waitUntil: 'networkidle' });
	const field = page.getByRole('textbox', { name: 'Username' });

	await field.fill('sari-w');
	await expect(page.getByText('Letters, numbers, underscores and periods only.')).toBeVisible();
	await field.fill(taken);
	await expect(page.getByText('Username is already taken')).toBeVisible();
	await expect(field).toHaveAttribute('aria-invalid', 'true');
});

test('the photo is changed and removed', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/profile', { waitUntil: 'networkidle' });
	await page.locator('input[type="file"]').setInputFiles({ name: 'me.png', mimeType: 'image/png', buffer: PNG });
	await expect(page.getByText('Photo saved')).toBeVisible();
	await expect(page.locator('header img')).toBeVisible();
	const profile = (await (await api(account, '/api/profile')).json()) as { avatar_url: string | null };
	expect(profile.avatar_url).toMatch(/^\/avatars\//);

	await page.getByRole('button', { name: 'Remove', exact: true }).click();
	await expect(page.getByText('Photo removed')).toBeVisible();
	await expect(page.locator('header img')).toHaveCount(0);
});
