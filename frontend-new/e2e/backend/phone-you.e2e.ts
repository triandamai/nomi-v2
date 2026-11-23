import { expect, test } from '@playwright/test';
import { BACKEND_URL } from './support/env';
import { api, createAccount, newEmail, signInAs, type Account } from './support/accounts';
import { codeFor } from './support/mailpit';
import { seedMemory, seedNomiModel, seedPersonality, seedUsage } from './support/seed';
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

test('Models & providers on a phone: defaults from sheets, a provider’s switches, Connect as a sheet, the key on its own screen', async ({ page, context }) => {
	const account = await createAccount();
	await seedNomiModel(account.email);
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('main').getByRole('link', { name: /^Models & providers/ }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Models & providers' })).toBeVisible();
	await expect(page.getByRole('banner')).toBeHidden();

	// Koda's model: a sheet, kept.
	await page.getByRole('button', { name: /^Koda builds with/ }).click();
	await page.getByRole('dialog', { name: 'Koda builds with' }).getByRole('button', { name: 'Same as new chats' }).click();
	type Models = { coding: { kind: string } };
	await expect.poll(async () => ((await (await api(account, '/api/llm/models')).json()) as Models).coding.kind).toBe('same_as_chat');
	await expect(page.getByRole('button', { name: /^Koda builds with Same as new chats/ })).toBeVisible();

	// Nomi's own models: their screen, a switch each, saved as it flips.
	await page.getByRole('region', { name: 'Your providers' }).getByRole('link').first().click();
	await expect(page).toHaveURL(/\?provider=/);
	const model = page.getByRole('switch', { name: 'E2E default (fake)' });
	await expect(model).toHaveAttribute('aria-checked', 'true');
	await model.click();
	await expect(model).toHaveAttribute('aria-checked', 'false');
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('switch', { name: 'E2E default (fake)' })).toHaveAttribute('aria-checked', 'false');
	await page.getByRole('switch', { name: 'E2E default (fake)' }).click();
	await expect(page.getByRole('switch', { name: 'E2E default (fake)' })).toHaveAttribute('aria-checked', 'true');
	// Nomi's own can't be disconnected.
	await expect(page.getByRole('button', { name: /^Disconnect/ })).toHaveCount(0);
	await page.getByRole('main').getByRole('link', { name: 'Models', exact: true }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);

	// Connect a provider: a sheet with search; a provider opens its key screen, without the tab bar.
	await page.getByRole('link', { name: /^Connect a provider/ }).click();
	await expect(page).toHaveURL(/\?connect$/);
	const sheet = page.getByRole('dialog', { name: 'Connect a provider' });
	await sheet.getByRole('searchbox', { name: 'Search providers' }).fill('anthro');
	await sheet.getByRole('button', { name: /^Anthropic/ }).first().click();
	await expect(page).toHaveURL(/\?connect=anthropic$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Anthropic' })).toBeVisible();
	await expect(page.getByLabel('API key')).toHaveAttribute('type', 'password');
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeHidden();
	await expect(page.getByRole('button', { name: 'Check and connect' })).toBeDisabled();
	await page.getByRole('button', { name: 'Models', exact: true }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);

	// A custom endpoint nobody answers at: turned away with the reason, and nothing kept.
	await page.getByRole('link', { name: /^Connect a provider/ }).click();
	await page.getByRole('dialog', { name: 'Connect a provider' }).getByRole('button', { name: /^Custom endpoint/ }).click();
	await page.getByLabel('Address').fill('https://127.0.0.1:9/v1');
	await page.getByLabel('Address').press('Enter');
	await expect(page.getByLabel('Address')).toHaveAttribute('aria-invalid', 'true', { timeout: 30_000 });
	await page.getByRole('button', { name: 'Models', exact: true }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);
	const { connections } = (await (await api(account, '/api/llm/connections')).json()) as { connections: unknown[] };
	expect(connections).toEqual([]);
});

test('Memory on a phone: the profile on its own screen, a kind’s memories, correct and forget with Undo, budgets from a sheet', async ({ page, context }) => {
	const account = await createAccount();
	await seedMemory(account.email, { content: 'Short, direct answers', kind: 'preference', weight: 4 });
	await seedMemory(account.email, { content: 'Siti is my sister in Bandung', kind: 'person' });
	await seedMemory(account.email, { content: 'Pak Budi supplies the beans', kind: 'person' });
	type Memory = { id: string; content: string };
	const memories = async () => ((await (await api(account, '/api/memory')).json()) as { memories: Memory[] }).memories;
	await signInAs(context, account);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('main').getByRole('link', { name: /^Memory/ }).click();
	await expect(page).toHaveURL(/\/settings\/memory$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Memory & context' })).toBeVisible();
	await expect(page.getByRole('link', { name: /^People 2/ })).toBeVisible();

	// The profile: its own screen, saved, back with it shown.
	await page.getByRole('button', { name: 'Edit the profile' }).click();
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeHidden();
	await page.getByRole('textbox', { name: 'Your profile' }).fill('Runs a coffee stall in Bandung.');
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByText('Profile saved. The crew reads it from the next reply.')).toBeVisible();
	await expect(page.getByText('Runs a coffee stall in Bandung.')).toBeVisible();
	await expect.poll(async () => ((await (await api(account, '/api/memory/profile')).json()) as { text: string }).text).toBe('Runs a coffee stall in Bandung.');

	// People: each memory a card; correct one on its own screen.
	await page.getByRole('link', { name: /^People/ }).click();
	await expect(page).toHaveURL(/\?topic=person$/);
	await expect(page.getByRole('heading', { level: 1, name: 'People' })).toBeVisible();
	await page.getByRole('article', { name: 'Siti is my sister in Bandung' }).getByRole('button', { name: 'More for this memory' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Correct it' }).click();
	await page.getByRole('textbox', { name: 'What Nomi should remember' }).fill('Siti is my sister in Jakarta');
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByRole('article', { name: 'Siti is my sister in Jakarta' })).toBeVisible();
	await expect.poll(async () => (await memories()).map((one) => one.content)).toContain('Siti is my sister in Jakarta');

	// Forget offers Undo: undone, it stays; forgotten, it goes.
	const beans = page.getByRole('article', { name: 'Pak Budi supplies the beans' });
	await beans.getByRole('button', { name: 'More for this memory' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Forget it' }).click();
	await expect(beans).toBeHidden();
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(beans).toBeVisible();
	await page.waitForTimeout(6_500);
	expect((await memories()).map((one) => one.content)).toContain('Pak Budi supplies the beans');
	await beans.getByRole('button', { name: 'More for this memory' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Forget it' }).click();
	await expect(beans).toBeHidden();
	await expect.poll(async () => (await memories()).map((one) => one.content), { timeout: 15_000 }).not.toContain('Pak Budi supplies the beans');

	// Back to Memory: a budget from a sheet, a learning switch saved as it flips.
	await page.getByRole('main').getByRole('link', { name: 'Memory', exact: true }).click();
	await page.getByRole('button', { name: /^Profile .* tok$/ }).click();
	await page.getByRole('dialog', { name: 'Profile' }).getByRole('button', { name: '2,000 tok' }).click();
	const settings = async () => (await (await api(account, '/api/memory/settings')).json()) as { search_past_chats: boolean; profile_budget: number };
	await expect.poll(async () => (await settings()).profile_budget).toBe(2000);
	const search = page.getByRole('switch', { name: 'Search past chats' });
	const before = (await settings()).search_past_chats;
	await search.click();
	await expect.poll(async () => (await settings()).search_past_chats).toBe(!before);
});

type Orgs = { orgs: { id: string; name: string }[]; project_org_id: string | null };
type OrgDetail = { name: string; members: { user_id: string; name: string; role: string }[]; invites: { code: string; email: string }[] };
const orgsOf = async (account: Account) => (await (await api(account, '/api/orgs')).json()) as Orgs;
const orgOf = async (account: Account, id: string) => (await (await api(account, `/api/orgs/${id}`)).json()) as OrgDetail;

test('Workspaces on a phone: create on its own screen, where projects go from a sheet, invite from a sheet, roles and Remove', async ({ page, context }) => {
	const owner = await createAccount();
	const siti = await createAccount();
	await signInAs(context, owner);
	await page.goto('/settings', { waitUntil: 'networkidle' });
	await page.getByRole('main').getByRole('link', { name: /^Workspaces/ }).click();
	await expect(page).toHaveURL(/\/settings\/workspaces$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Workspaces' })).toBeVisible();

	// Create: its own screen, without the tab bar.
	await page.getByRole('button', { name: 'Create a workspace' }).click();
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeHidden();
	await page.getByRole('textbox', { name: 'Name' }).fill('Warung Dua');
	await page.getByRole('textbox', { name: 'Name' }).press('Enter');
	await expect(page.getByText('Created Warung Dua')).toBeVisible();
	await expect(page.getByRole('link', { name: /^Warung Dua/ })).toBeVisible();
	const made = (await orgsOf(owner)).orgs.find((org) => org.name === 'Warung Dua')!;

	// Where new projects go: a sheet.
	await page.getByRole('button', { name: /^New projects go to/ }).click();
	await page.getByRole('dialog', { name: 'New projects go to' }).getByRole('button', { name: /^Warung Dua/ }).click();
	await expect.poll(async () => (await orgsOf(owner)).project_org_id).toBe(made.id);

	// The first workspace: invite Siti as an admin from the sheet.
	const kopi = (await orgsOf(owner)).orgs.find((org) => org.id !== made.id)!;
	await page.goto(`/settings/workspaces/${kopi.id}`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: kopi.name })).toBeVisible();
	await expect(page.getByRole('main').getByRole('link', { name: 'Workspaces', exact: true })).toHaveAttribute('href', '/settings/workspaces');
	await page.getByRole('button', { name: /^Invite people/ }).click();
	const sheet = page.getByRole('dialog', { name: `Invite to ${kopi.name}` });
	await sheet.getByRole('textbox', { name: 'Email addresses' }).fill(siti.email);
	await sheet.getByRole('button', { name: /^Admin/ }).click();
	await sheet.getByRole('button', { name: 'Send invite' }).click();
	await expect(page.getByText('Invite sent')).toBeVisible();
	await expect(sheet).toBeHidden();

	// An open invite: Copy link or Revoke. A stranger's goes.
	const stranger = newEmail();
	await api(owner, `/api/orgs/${kopi.id}/invites`, { method: 'POST', body: JSON.stringify({ email: stranger, role: 'member' }) });
	await page.reload({ waitUntil: 'networkidle' });
	await page.getByRole('button', { name: new RegExp(`^${stranger.replace(/[.+]/g, '\\$&')}`) }).click();
	await page.getByRole('dialog', { name: stranger }).getByRole('button', { name: 'Revoke' }).click();
	await expect.poll(async () => (await orgOf(owner, kopi.id)).invites.map((one) => one.email)).not.toContain(stranger);

	// Siti joins; her role from a sheet, then Remove, which asks first.
	const code = (await orgOf(owner, kopi.id)).invites.find((one) => one.email === siti.email)!.code;
	expect((await api(siti, `/api/invites/${code}/accept`, { method: 'POST' })).ok).toBe(true);
	await page.reload({ waitUntil: 'networkidle' });
	const sitiName = (await orgOf(owner, kopi.id)).members.find((one) => one.role === 'admin')!.name;
	await page.getByRole('button', { name: new RegExp(`^${sitiName} Admin`) }).click();
	await page.getByRole('dialog', { name: sitiName }).getByRole('button', { name: 'Member' }).click();
	await expect.poll(async () => (await orgOf(owner, kopi.id)).members.find((one) => one.name === sitiName)?.role).toBe('member');
	await page.getByRole('button', { name: new RegExp(`^${sitiName} Member`) }).click();
	await page.getByRole('dialog', { name: sitiName }).getByRole('button', { name: `Remove ${sitiName}` }).click();
	await page.getByRole('dialog', { name: `Remove ${sitiName}?` }).getByRole('button', { name: 'Remove', exact: true }).click();
	await expect.poll(async () => (await orgOf(owner, kopi.id)).members.length).toBe(1);

	// The only owner can't leave: the row says why and does nothing.
	await expect(page.getByText('You’re its only owner: make someone else an owner first.')).toBeVisible();

	// Rename, off ⋯, on its own screen.
	await page.getByRole('button', { name: `More for ${kopi.name}` }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Rename' }).click();
	await page.getByRole('textbox', { name: 'Name' }).fill('Kopi Kita');
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Kopi Kita' })).toBeVisible();
	await expect.poll(async () => (await orgOf(owner, kopi.id)).name).toBe('Kopi Kita');
});

test('a member leaves a workspace on a phone, after it asks', async ({ page, context }) => {
	const owner = await createAccount();
	const budi = await createAccount();
	const kopi = (await orgsOf(owner)).orgs[0];
	const invited = (await (await api(owner, `/api/orgs/${kopi.id}/invites`, { method: 'POST', body: JSON.stringify({ email: budi.email, role: 'member' }) })).json()) as { code: string };
	expect((await api(budi, `/api/invites/${invited.code}/accept`, { method: 'POST' })).ok).toBe(true);
	await signInAs(context, budi);
	await page.goto(`/settings/workspaces/${kopi.id}`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('button', { name: /^Invite people/ })).toHaveCount(0);
	await expect(page.getByRole('button', { name: `More for ${kopi.name}` })).toHaveCount(0);
	await page.getByRole('button', { name: `Leave ${kopi.name}` }).click();
	await page.getByRole('dialog', { name: `Leave ${kopi.name}` }).getByRole('button', { name: 'Leave' }).click();
	await expect(page).toHaveURL(/\/settings\/workspaces$/);
	await expect(page.getByText(`You left ${kopi.name}`)).toBeVisible();
	expect((await orgsOf(budi)).orgs.map((org) => org.id)).not.toContain(kopi.id);
});
