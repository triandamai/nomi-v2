import { expect, test } from '../support/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { seedNotification } from './support/seed';

// Home, Inbox and You on a phone against the real backend (Flow / Mobile — Home, Inbox & You):
// each tab's own screen with its large title, and what it does.

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

type Reminder = { title: string; status: string };
async function reminders(account: Account): Promise<Reminder[]> {
	const list = (await (await api(account, '/api/reminders')).json()) as { upcoming: Reminder[]; past: Reminder[] };
	return [...list.upcoming, ...list.past];
}

test('Home on a phone: the greeting, the ask card starting a chat, who it goes to, and its + opening a new chat', async ({ page, context }) => {
	const account = await createAccount();
	// The greeting names whoever is signed in, by their first name.
	expect((await api(account, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Budi Santoso' }) })).ok).toBe(true);
	await signInAs(context, account);
	await page.goto('/', { waitUntil: 'networkidle' });

	await expect(page.getByRole('heading', { level: 1 })).toHaveText(/\bBudi\.$/);
	await expect(page.getByRole('heading', { level: 1 })).not.toContainText('Santoso');
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Home' })).toHaveAttribute('aria-current', 'page');
	await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
	await expect(page.getByRole('banner')).toBeHidden();
	await expect(page.getByRole('link', { name: 'New chat' })).toHaveAttribute('href', '/chat/new');

	// Who it goes to is a sheet; Nomi is there and chosen.
	await page.getByRole('button', { name: /^Send to .+, change$/ }).click();
	const sheet = page.getByRole('dialog', { name: 'Send to' });
	await expect(sheet.getByRole('button', { name: 'Nomi' })).toBeVisible();
	await sheet.getByRole('button', { name: 'Nomi' }).click();
	await expect(sheet).toHaveCount(0);

	// Asking starts a chat and opens it.
	await page.getByPlaceholder('Ask Nomi, or hand something to the crew…').fill('What can I build in 10 minutes?');
	await page.getByRole('button', { name: 'Send', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]{36}$/);
});

test('the Inbox on a phone: the line under the title, cards that need the person, Mark all read, and New reminder in a sheet', async ({ page, context }) => {
	const account = await createAccount();
	await seedNotification(account.email, { title: 'Your plan is now Pro', body: '1M tokens a month from today.' });
	await seedNotification(account.email, { title: 'Your chats moved', body: 'See them all in one place.', link: '/chats' });
	await seedNotification(account.email, { title: 'Usage at 80%', body: 'You have 200k tokens left this month.' });
	await signInAs(context, account);
	await page.goto('/inbox', { waitUntil: 'networkidle' });

	await expect(page.getByRole('heading', { level: 1, name: 'Inbox' })).toBeVisible();
	await expect(page.getByRole('banner')).toBeHidden();
	await expect(page.getByRole('main').getByText('3 need you · 0 coming up this week')).toBeVisible();
	await expect(page.getByRole('radio', { name: /Needs you 3/ })).toBeVisible();

	// Mark read on a card takes it out of Needs you.
	const plan = page.getByRole('article', { name: 'Your plan is now Pro' });
	await plan.getByRole('button', { name: 'Mark read' }).click();
	await expect(plan).toBeHidden();
	await expect(page.getByRole('main').getByText('2 need you · 0 coming up this week')).toBeVisible();

	// Mark all read, from beside the title: the all-clear state, as C6vql9 draws it.
	await page.getByRole('button', { name: 'Mark all read' }).click();
	await expect(page.getByText('All marked read')).toBeVisible();
	await expect(page.getByText('Approvals, invites and anything the crew can’t finish alone land here first.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Mark all read' })).toBeDisabled();
	expect(((await (await api(account, '/api/notifications')).json()) as { unread: number }).unread).toBe(0);

	// New reminder floats over the tab bar and opens the quick reminder in a sheet.
	await page.getByRole('button', { name: 'New reminder' }).click();
	const sheet = page.getByRole('dialog', { name: 'New reminder' });
	const line = sheet.getByRole('textbox', { name: 'Quick reminder' });
	await line.fill('stretch tomorrow at 4pm');
	await line.press('Enter');
	await expect(page.getByText(/^Reminder set for .+ · 16:00$/)).toBeVisible();
	await expect(sheet).toHaveCount(0);
	await expect.poll(async () => (await reminders(account)).map((one) => [one.title, one.status])).toEqual([['Stretch', 'active']]);
	await expect(page.getByRole('main').getByText('Stretch', { exact: true })).toBeVisible();
});

test('You on a phone: who’s signed in, the plan, the settings in their groups, back, and Log out', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'You' }).click();
	await expect(page).toHaveURL(/\/settings$/);

	await expect(page.getByRole('heading', { level: 1, name: 'You' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'You' })).toHaveAttribute('aria-current', 'page');
	await expect(page.getByRole('link', { name: /Profile/ }).first()).toHaveAttribute('href', '/settings/profile');
	await expect(page.getByRole('meter')).toBeVisible();
	await expect(page.getByRole('link', { name: 'Plan & usage' })).toHaveAttribute('href', '/settings/plan');
	for (const group of ['Account', 'AI', 'Workspace']) await expect(page.getByRole('region', { name: group })).toBeVisible();

	// A settings page is a screen inside You, with the way back.
	await page.getByRole('region', { name: 'AI' }).getByRole('link', { name: /Crew/ }).click();
	await expect(page).toHaveURL(/\/settings\/crew$/);
	await page.getByRole('main').getByRole('link', { name: 'You', exact: true }).click();
	await expect(page).toHaveURL(/\/settings$/);

	await page.getByRole('button', { name: 'Log out' }).click();
	await expect(page).toHaveURL(/\/sign-in/);
});

test.describe('on a desktop', () => {
	test.use({ viewport: { width: 1512, height: 862 }, userAgent: undefined, hasTouch: false });
	test('Settings still opens on Profile', async ({ page, context }) => {
		const account = await createAccount();
		await signInAs(context, account);
		await page.goto('/settings', { waitUntil: 'networkidle' });
		await expect(page).toHaveURL(/\/settings\/profile$/);
	});
});
