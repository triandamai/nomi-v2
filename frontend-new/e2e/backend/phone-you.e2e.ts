import { expect, test } from '@playwright/test';
import { BACKEND_URL } from './support/env';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { codeFor } from './support/mailpit';

// The You flow on a phone against the real backend (Flow / Mobile — You & Settings, EtSi2): each
// settings page is its own screen pushed from You, its rows show what's set, picking opens a sheet,
// typing gets its own screen, and every change reaches the backend.

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

type Preferences = { theme: string; timezone: string; language: string; default_agent: string | null; enter_sends: boolean };
const preferences = async (account: Account) => (await (await api(account, '/api/preferences')).json()) as Preferences;
const reminders = async (account: Account) => (await (await api(account, '/api/notifications/reminder-preferences')).json()) as { push_reminders: boolean; quiet_hours: { start: string; end: string } | null };
const login = (email: string, password: string) => fetch(`${BACKEND_URL}/api/auth/login`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ email, password }) });

test('General on a phone: rows show what’s set, sheets pick, switches save', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('region', { name: 'Account' }).getByRole('link', { name: /General/ }).click();
	await expect(page).toHaveURL(/\/settings\/general$/);
	await expect(page.getByRole('main').getByRole('link', { name: 'You' })).toHaveAttribute('href', '/settings');
	await expect(page.getByRole('banner')).toBeHidden();
	await expect(page.getByRole('button', { name: new RegExp(`Email ${account.email.slice(0, 2)}`) })).toBeVisible();

	// Theme: a sheet, the chosen one ticked; picking sets it at once and keeps it.
	await page.getByRole('button', { name: /^Theme/ }).click();
	const themes = page.getByRole('dialog', { name: 'Theme' });
	await expect(themes.getByRole('button', { name: 'System' })).toBeVisible();
	await themes.getByRole('button', { name: 'Dark' }).click();
	await expect(themes).toBeHidden();
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
	await expect.poll(async () => (await preferences(account)).theme).toBe('dark');
	await expect(page.getByRole('button', { name: /^Theme Dark/ })).toBeVisible();

	// Time zone and who chats start with: sheets too.
	await page.getByRole('button', { name: /^Time zone/ }).click();
	await page.getByRole('dialog', { name: 'Time zone' }).getByRole('button', { name: /Asia\/Makassar/ }).click();
	await expect.poll(async () => (await preferences(account)).timezone).toBe('Asia/Makassar');
	await page.getByRole('button', { name: /^New chats start with/ }).click();
	await page.getByRole('dialog', { name: 'New chats start with' }).getByRole('button', { name: 'Dana' }).click();
	await expect.poll(async () => (await preferences(account)).default_agent).toBe('money');

	// Switches save as they flip.
	await page.getByRole('switch', { name: 'Enter sends' }).click();
	await expect.poll(async () => (await preferences(account)).enter_sends).toBe(false);
	await page.getByRole('switch', { name: 'Product news' }).click();
	await expect.poll(async () => ((await (await api(account, '/api/notifications/preferences')).json()) as { email_promos: boolean }).email_promos).toBe(false);

	// Quiet hours: Off, then a night, then Off again.
	await page.getByRole('button', { name: /^Quiet hours Off/ }).click();
	await page.getByRole('dialog', { name: 'Quiet hours' }).getByRole('button', { name: /23:00/ }).click();
	await expect.poll(async () => (await reminders(account)).quiet_hours).toEqual({ start: '23:00', end: '07:00' });
	await page.getByRole('button', { name: /^Quiet hours 23:00/ }).click();
	await page.getByRole('dialog', { name: 'Quiet hours' }).getByRole('button', { name: 'Off' }).click();
	await expect.poll(async () => (await reminders(account)).quiet_hours).toBeNull();

	// Language: the page comes back in it.
	await page.getByRole('button', { name: /^Language/ }).click();
	await page.getByRole('dialog', { name: 'Language' }).getByRole('button', { name: 'Bahasa Indonesia' }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Umum' })).toBeVisible();
	expect((await preferences(account)).language).toBe('id');
});

test('the email and the password change on their own screens, with the tab bar out of the way', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });

	// Password: its own screen; the keyboard's return saves.
	await page.getByRole('button', { name: /^Password Change/ }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Change your password' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' })).toHaveCount(0);
	await page.getByLabel('Current password').fill(account.password);
	await page.getByLabel('New password', { exact: true }).fill('brand-new-horse');
	await page.getByLabel('Confirm new password').fill('brand-new-horse');
	await page.getByLabel('Confirm new password').press('Enter');
	await expect(page.getByText('Password changed. Your other devices stay signed in.')).toBeVisible();
	await expect(page.getByRole('heading', { level: 1, name: 'General' })).toBeVisible();
	expect((await login(account.email, 'brand-new-horse')).status).toBe(200);

	// Email: the address, then the code, then it's the one that signs in.
	await page.getByRole('button', { name: /^Email/ }).first().click();
	await expect(page.getByRole('heading', { level: 1, name: 'Change your email' })).toBeVisible();
	const send = page.getByRole('button', { name: 'Send code' });
	await expect(send).toBeDisabled();
	const next = `moved-${Date.now()}@nomi.test`;
	await page.getByLabel('New email').fill(next);
	const started = Date.now();
	await send.click();
	await expect(page.getByRole('heading', { level: 1, name: 'Check your new email' })).toBeVisible();
	const code = await codeFor(next, /new email/i, { after: started - 1000 });
	await page.getByRole('textbox', { name: 'Digit 1 of 6' }).click();
	await page.keyboard.type(code);
	await expect(page.getByText(`Your email is now ${next}. Sign in with it from now on.`)).toBeVisible();
	expect((await login(next, 'brand-new-horse')).status).toBe(200);
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
});

test('Delete account is last, alone, and closes the account from its sheet', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });
	await page.getByRole('button', { name: 'Delete account' }).click();
	const sheet = page.getByRole('dialog', { name: 'Delete your account?' });
	await sheet.getByRole('button', { name: 'Keep my account' }).click();
	await expect(sheet).toBeHidden();
	await page.getByRole('button', { name: 'Delete account' }).click();
	await sheet.getByLabel('Password').fill(account.password);
	await sheet.getByRole('button', { name: 'Delete account' }).click();
	await expect(page).toHaveURL(/\/sign-in\?closed=1$/);
	expect((await login(account.email, account.password)).status).toBe(401);
});

const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==', 'base64');
const unique = () => `e2e_${Date.now().toString(36)}${Math.floor(Math.random() * 1000)}`;
type Profile = { display_name: string | null; username: string | null; avatar_url: string | null; show_stats_publicly: boolean; show_crew_publicly: boolean };
const profileOf = async (account: Account) => (await (await api(account, '/api/profile')).json()) as Profile;

test('Profile on a phone: each field on its own screen with its own Save, switches save as they flip', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('link', { name: /Profile/ }).first().click();
	await expect(page).toHaveURL(/\/settings\/profile$/);
	await expect(page.getByRole('main').getByRole('link', { name: 'You' })).toHaveAttribute('href', '/settings');
	await expect(page.getByRole('img', { name: /tokens in the last 13 weeks/ })).toBeVisible();
	await expect(page.getByRole('term').filter({ hasText: 'Current streak' })).toBeVisible();
	// No unsaved-changes bar on a phone.
	await expect(page.getByRole('button', { name: 'Discard' })).toHaveCount(0);

	// The display name: its own screen, without the tab bar; Save takes it back with the name shown.
	await page.getByRole('button', { name: /^Display name/ }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Display name' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeHidden();
	await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();
	await page.getByRole('textbox', { name: 'Display name' }).fill('Sari Wulandari');
	await page.getByRole('textbox', { name: 'Display name' }).press('Enter');
	await expect(page.getByText('Profile saved')).toBeVisible();
	await expect(page.getByRole('heading', { level: 1, name: 'Sari Wulandari' })).toBeVisible();
	await expect(page.getByRole('button', { name: /^Display name Sari Wulandari/ })).toBeVisible();
	await expect.poll(async () => (await profileOf(account)).display_name).toBe('Sari Wulandari');

	// The username: checked as it's typed; back without saving leaves it as it was.
	await page.getByRole('button', { name: /^Username/ }).click();
	await page.getByRole('textbox', { name: 'Username' }).fill('sari-w');
	await expect(page.getByText('Letters, numbers, underscores and periods only.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();
	await page.getByRole('button', { name: 'Profile' }).click();
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
	const username = unique();
	await page.getByRole('button', { name: /^Username/ }).click();
	await page.getByRole('textbox', { name: 'Username' }).fill(username);
	await expect(page.getByText(`/@${username} · available`)).toBeVisible();
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByText(`@${username} · joined`)).toBeVisible();
	await expect.poll(async () => (await profileOf(account)).username).toBe(username);

	// The public switches save as they flip.
	await page.getByRole('switch', { name: 'Show crew on profile' }).click();
	await expect.poll(async () => (await profileOf(account)).show_crew_publicly).toBe(true);
	await page.getByRole('switch', { name: 'Show crew on profile' }).click();
	await expect.poll(async () => (await profileOf(account)).show_crew_publicly).toBe(false);

	// Showcase: nothing live yet, so nothing to add, and why.
	await expect(page.getByText('Publish a project to feature it here.')).toBeVisible();
});

test('Profile on a phone: the photo is changed from the header and removed from ⋯', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/profile', { waitUntil: 'networkidle' });
	await expect(page.getByRole('button', { name: 'More for your profile' })).toHaveCount(0);
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Change photo' }).click();
	await (await chooser).setFiles({ name: 'me.png', mimeType: 'image/png', buffer: PNG });
	await expect(page.getByText('Photo saved')).toBeVisible();
	await expect.poll(async () => (await profileOf(account)).avatar_url).toMatch(/^\/avatars\//);

	await page.getByRole('button', { name: 'More for your profile' }).click();
	await page.getByRole('dialog', { name: 'Photo' }).getByRole('button', { name: 'Remove' }).click();
	await expect(page.getByText('Photo removed')).toBeVisible();
	await expect.poll(async () => (await profileOf(account)).avatar_url).toBeNull();
});
