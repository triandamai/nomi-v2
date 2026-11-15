import { randomUUID } from 'node:crypto';
import { expect, test } from '@playwright/test';
import { BACKEND_URL } from './support/env';
import { api, createAccount, signInAs } from './support/accounts';

// Settings › General against the real backend (App / Settings / General m7lvY1): each change
// saves as it's made (theme, time zone, emails, language), the password is changed in place (the
// emailed reset for a forgotten one), and other devices are signed out.

type Preferences = { theme: string; timezone: string; language: string; has_stored_timezone: boolean; default_agent: string | null; enter_sends: boolean };
const preferences = async (account: Parameters<typeof api>[0]) => (await (await api(account, '/api/preferences')).json()) as Preferences;

test('general settings save as they’re changed', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/profile', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'General' }).click();
	await expect(page).toHaveURL(/\/settings\/general$/);
	await expect(page.getByRole('banner').getByText('changes save automatically')).toBeVisible();
	await expect(page.getByText(`${account.email.slice(0, 2)}•`)).toBeVisible();

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

test('the password changes in place: the current one first, then the new one twice', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });
	await page.getByRole('button', { name: 'Change', exact: true }).click();
	const dialog = page.getByRole('dialog', { name: 'Change your password' });
	await expect(dialog.getByText('Your other devices stay signed in.')).toBeVisible();
	await expect(dialog.getByRole('link', { name: 'Forgot it? Reset it by email' })).toHaveAttribute('href', `/forgot-password?email=${encodeURIComponent(account.email)}`);

	await dialog.getByLabel('Current password').fill('not-my-password');
	await dialog.getByLabel('New password', { exact: true }).fill('brand-new-horse');
	await dialog.getByLabel('Confirm new password').fill('brand-new-hors');
	await dialog.getByRole('button', { name: 'Save password' }).click();
	await expect(dialog.getByText('The two passwords don’t match.')).toBeVisible();
	await dialog.getByLabel('Confirm new password').fill('brand-new-horse');
	await dialog.getByRole('button', { name: 'Save password' }).click();
	await expect(dialog.getByText('That isn’t your current password.')).toBeVisible();

	await dialog.getByLabel('Current password').fill(account.password);
	await dialog.getByLabel('Current password').press('Enter');
	await expect(page.getByText('Password changed. Your other devices stay signed in.')).toBeVisible();
	await expect(dialog).toHaveCount(0);

	// The new one is the one that works now; this device is still signed in.
	const login = (password: string) =>
		fetch(`${BACKEND_URL}/api/auth/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ email: account.email, password }) });
	expect((await login(account.password)).status).toBe(401);
	expect((await login('brand-new-horse')).status).toBe(200);
	await page.reload();
	await expect(page).toHaveURL(/\/settings\/general$/);
});


test('who new chats start with and Enter sends reach Home; reminder pushes and quiet hours are kept', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });

	// Chat.
	await page.getByRole('combobox', { name: 'New chats start with' }).selectOption({ label: 'Dana' });
	await expect.poll(async () => (await preferences(account)).default_agent).toBe('money');
	await page.getByRole('switch', { name: 'Enter sends' }).click();
	await expect(page.getByText('Enter adds a new line; ⌘ or Ctrl+Enter sends')).toBeVisible();
	await expect.poll(async () => (await preferences(account)).enter_sends).toBe(false);

	// Notifications: pushes off, then quiet hours on, a later night, and off again.
	const reminders = async () =>
		(await (await api(account, '/api/notifications/reminder-preferences')).json()) as { push_reminders: boolean; quiet_hours: { start: string; end: string } | null };
	await page.getByRole('switch', { name: 'Reminders' }).click();
	await expect.poll(async () => (await reminders()).push_reminders).toBe(false);
	const quiet = page.getByRole('combobox', { name: 'Quiet hours' });
	await expect(quiet).toBeDisabled();
	await page.getByRole('switch', { name: 'Quiet hours' }).click();
	await expect.poll(async () => (await reminders()).quiet_hours).toEqual({ start: '22:00', end: '06:00' });
	await expect(page.getByText('22:00–06:00 · reminders wait until they end.')).toBeVisible();
	await quiet.selectOption('23:00-07:00');
	await expect.poll(async () => (await reminders()).quiet_hours).toEqual({ start: '23:00', end: '07:00' });
	await page.reload({ waitUntil: 'networkidle' });
	await expect(quiet).toHaveValue('23:00-07:00');
	await page.getByRole('switch', { name: 'Quiet hours' }).click();
	await expect.poll(async () => (await reminders()).quiet_hours).toBeNull();

	// Home: the message goes to Dana unless picked otherwise, and Enter makes a new line.
	await page.goto('/', { waitUntil: 'networkidle' });
	await expect(page.getByRole('button', { name: /Dana/ }).first()).toBeVisible();
	const field = page.getByRole('textbox', { name: 'Message' });
	await field.fill('Two');
	await field.press('Enter');
	await expect(field).toHaveValue('Two\n');
	await expect(page).toHaveURL(/\/$/);
});
