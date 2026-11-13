import { randomUUID } from 'node:crypto';
import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Settings › General against the real backend (App / Settings / General m7lvY1): each change
// saves as it's made (theme, time zone, emails, language), the password goes through the emailed
// reset, and other devices are signed out.

type Preferences = { theme: string; timezone: string; language: string; has_stored_timezone: boolean };
const preferences = async (account: Parameters<typeof api>[0]) => (await (await api(account, '/api/preferences')).json()) as Preferences;

test('general settings save as they’re changed', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/profile', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'General' }).click();
	await expect(page).toHaveURL(/\/settings\/general$/);
	await expect(page.getByRole('banner').getByText('changes save automatically')).toBeVisible();
	await expect(page.getByText(`${account.email.slice(0, 2)}•`)).toBeVisible();
	await expect(page.getByRole('link', { name: 'Change' })).toHaveAttribute('href', `/forgot-password?email=${encodeURIComponent(account.email)}`);

	// Theme: on the page at once, and kept.
	await page.getByRole('radio', { name: 'Light' }).click();
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	await expect.poll(async () => (await preferences(account)).theme).toBe('light');

	// Time zone: another one (sign-up saves the first).
	await page.getByRole('combobox', { name: 'Time zone' }).selectOption('Asia/Makassar');
	await expect.poll(async () => (await preferences(account)).timezone).toBe('Asia/Makassar');
	await expect(page.getByText('Used for reminders and your activity.')).toBeVisible();

	// Emails.
	await page.getByRole('switch', { name: 'Product news' }).click();
	await expect.poll(async () => ((await (await api(account, '/api/notifications/preferences')).json()) as { email_promos: boolean }).email_promos).toBe(false);

	// Language: the page comes back in it.
	await page.getByRole('radio', { name: 'Bahasa Indonesia' }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Umum' })).toBeVisible();
	expect((await preferences(account)).language).toBe('id');
});

test('the other devices are signed out, this one stays', async ({ page, context }) => {
	const account = await createAccount();
	const phone = randomUUID();
	await api(account, '/api/devices', {
		method: 'POST',
		headers: { 'x-device-id': phone },
		body: JSON.stringify({ device_id: phone, platform: 'ios', name: 'iPhone 15', app_version: '1.0.0' }),
	});
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });
	await expect(page.getByText(/iPhone 15/)).toBeVisible();

	await page.getByRole('button', { name: 'Sign out others' }).click();
	await expect(page.getByText(/^Signed out of \d+ other devices$/)).toBeVisible();
	await expect(page.getByText(/iPhone 15/)).toBeHidden();
	await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeVisible();
});
