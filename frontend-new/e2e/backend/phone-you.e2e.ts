import { expect, test } from '@playwright/test';
import { BACKEND_URL } from './support/env';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { codeFor } from './support/mailpit';
import { seedNomiModel, seedPersonality, seedUsage } from './support/seed';
import { readFile } from 'node:fs/promises';

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
	await expect(page.getByRole('main').getByRole('link', { name: 'You', exact: true })).toHaveAttribute('href', '/settings');
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
	await expect(page.getByRole('main').getByRole('link', { name: 'You', exact: true })).toHaveAttribute('href', '/settings');
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

test('Plan & usage on a phone: the plan first, Compare plans in a sheet, the month below, other months off ⋯', async ({ page, context }) => {
	const account = await createAccount();
	await seedUsage(account.email, { model: 'deepseek-flash', input: 3_000, output: 500, costUsd: 0.05 });
	await seedUsage(account.email, { model: 'gemini-flash', input: 1_000, output: 200, costUsd: 0.01 });
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('link', { name: 'Plan & usage' }).click();
	await expect(page).toHaveURL(/\/settings\/plan$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Plan & usage' })).toBeVisible();
	await expect(page.getByRole('banner')).toBeHidden();
	await expect(page.getByRole('meter', { name: /^4\.7K \/ .+ tokens · \d+% used$/ })).toBeVisible();
	await expect(page.getByText(/^resets /)).toBeVisible();

	// The month: its figures, a bar a day, and by model.
	await expect(page.getByRole('definition').filter({ hasText: '$0.06' })).toBeVisible();
	await expect(page.getByRole('definition').filter({ hasText: 'across 2 models' })).toBeVisible();
	await expect(page.getByRole('list', { name: /^Each day of / })).toBeVisible();
	await expect(page.getByRole('region', { name: 'By model' }).getByText('deepseek-flash')).toBeVisible();
	await expect(page.getByRole('region', { name: 'By model' }).getByText('3.5K')).toBeVisible();

	// Compare plans: a sheet; Free is theirs, Pro has its waitlist, and joining it stays joined.
	await page.getByRole('button', { name: 'Compare plans' }).click();
	const plans = page.getByRole('dialog', { name: 'Compare plans' });
	await expect(plans.getByRole('article', { name: 'Free' }).getByText('Your plan')).toBeVisible();
	const pro = plans.getByRole('article', { name: 'Pro' });
	await expect(pro.getByText('Coming soon')).toBeVisible();
	await pro.getByRole('button', { name: 'Join the waitlist' }).click();
	await expect(pro.getByText('You’re on the waitlist')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(plans).toBeHidden();

	// Other months: picked from a sheet; the month before has nothing used.
	await page.getByRole('button', { name: 'Other months' }).click();
	const months = page.getByRole('dialog', { name: 'Other months' });
	await expect(months.locator('button[aria-pressed="true"]')).toHaveCount(1);
	await months.locator('button[aria-pressed="false"]').first().click();
	await expect(page).toHaveURL(/\?month=\d{4}-\d{2}$/);
	await expect(page.getByRole('heading', { level: 2, name: /^Usage in / })).toBeVisible();
	await expect(page.getByText('nothing used yet')).toBeVisible();

	// Export CSV is a row, and saves the month shown.
	await page.getByRole('button', { name: 'Other months' }).click();
	await page.getByRole('dialog', { name: 'Other months' }).locator('button[aria-pressed]').first().click();
	await expect(page.getByRole('heading', { level: 2, name: 'This month' })).toBeVisible();
	const download = page.waitForEvent('download');
	await page.getByRole('link', { name: 'Export usage CSV' }).click();
	const csv = await readFile((await (await download).path())!, 'utf8');
	expect(csv).toContain('deepseek-flash,nomi,3500,1,0.05');
});

test('Crew on a phone: the roster as rows, a member on their own screen, their model from a sheet', async ({ page, context }) => {
	const account = await createAccount();
	await seedNomiModel(account.email);
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('main').getByRole('link', { name: /^Crew/ }).click();
	await expect(page).toHaveURL(/\/settings\/crew$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Crew' })).toBeVisible();
	await expect(page.getByRole('main').getByText(/^\d+ members/)).toBeVisible();
	await expect(page.getByRole('region', { name: 'Core' }).getByRole('link')).toHaveCount(2);

	// Koda: their own screen, back to Crew; Koda's model picked from a sheet and kept.
	await page.getByRole('region', { name: 'Specialists' }).getByRole('link', { name: /^Koda/ }).click();
	await expect(page).toHaveURL(/\?member=coding$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Koda' })).toBeVisible();
	await expect(page.getByRole('main').getByRole('link', { name: 'Crew', exact: true })).toHaveAttribute('href', '/settings/crew');
	await page.getByRole('button', { name: /^Model/ }).click();
	const sheet = page.getByRole('dialog', { name: 'Model' });
	await sheet.getByRole('button', { name: 'Same as new chats' }).click();
	await expect(page.getByText('Model saved')).toBeVisible();
	type Models = { coding: { kind: string } };
	await expect.poll(async () => ((await (await api(account, '/api/llm/models')).json()) as Models).coding.kind).toBe('same_as_chat');

	// Someone whose model follows the chat: no sheet, just why.
	await page.goto('/settings/crew?member=money', { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: 'Dana' })).toBeVisible();
	await expect(page.getByText('Works on the model of the chat it’s helping in.')).toBeVisible();
	await expect(page.getByRole('button', { name: /^Model/ })).toHaveCount(0);

	// Personality: an earlier version brought back from its sheet.
	await seedPersonality(account.email, ['Friendly and chatty', 'Formal, full sentences']);
	await page.goto('/settings/crew?member=personality', { waitUntil: 'networkidle' });
	await expect(page.getByRole('region', { name: 'Versions · 2' })).toBeVisible();
	await page.getByRole('button', { name: /^v1/ }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Restore', exact: true }).click();
	await expect(page.getByText('Personality v1 is back')).toBeVisible();
	await expect(page.getByRole('region', { name: 'Versions · 3' })).toBeVisible();

	// Back to Crew, then to You.
	await page.getByRole('main').getByRole('link', { name: 'Crew', exact: true }).click();
	await expect(page).toHaveURL(/\/settings\/crew$/);
	await page.getByRole('main').getByRole('link', { name: 'You', exact: true }).click();
	await expect(page).toHaveURL(/\/settings$/);
});
