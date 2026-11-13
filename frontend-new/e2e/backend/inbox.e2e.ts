import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { fireReminder, seedNotification } from './support/seed';

// The Inbox against the real backend (App / Inbox BFoiZ): what needs the person (unread
// notifications, a reminder that went off), reading and opening them, the quick reminder read from
// one line, cancelling with undo, the filters, and which emails they get.

type Reminder = { id: string; title: string; status: string; recurrence: string | null };

async function reminders(account: Account): Promise<Reminder[]> {
	const list = (await (await api(account, '/api/reminders')).json()) as { upcoming: Reminder[]; past: Reminder[] };
	return [...list.upcoming, ...list.past];
}

const quick = (page: Page) => page.getByRole('textbox', { name: 'Quick reminder' });
// A row of the feed (Coming up lists the same titles).
const row = (page: Page, title: string) => page.locator('li[data-entry]').filter({ has: page.getByText(title, { exact: true }) });

test('notifications that need the person are read, opened and all marked read', async ({ page, context }) => {
	const account = await createAccount();
	await seedNotification(account.email, { title: 'Your plan is now Pro', body: '1M tokens a month from today.' });
	await seedNotification(account.email, { title: 'Your chats moved', body: 'See them all in one place.', link: '/chats' });
	await seedNotification(account.email, { kind: 'promo', title: 'Nomi speaks Javanese now', body: 'Try it in any chat.', read: true });
	await signInAs(context, account);
	await page.goto('/inbox', { waitUntil: 'networkidle' });

	await expect(page.getByRole('banner').getByText('2 need you · 0 coming up this week')).toBeVisible();
	await expect(page.getByRole('radio', { name: /Needs you 2/ })).toBeVisible();
	const plan = page.getByRole('article', { name: 'Your plan is now Pro' });
	await expect(plan.getByText('1M tokens a month from today.')).toBeVisible();
	await expect(row(page, 'Nomi speaks Javanese now')).toBeVisible();

	await plan.getByRole('button', { name: 'Mark read' }).click();
	await expect(plan).toBeHidden();
	await expect(page.getByRole('radio', { name: /Needs you 1/ })).toBeVisible();

	// Opening one marks it read and goes where it points.
	await page.getByRole('article', { name: 'Your chats moved' }).getByRole('button', { name: 'Open' }).click();
	await expect(page).toHaveURL(/\/chats$/);
	const after = (await (await api(account, '/api/notifications')).json()) as { unread: number };
	expect(after.unread).toBe(0);

	// Mark all read, from the top bar.
	await seedNotification(account.email, { title: 'Usage at 80%', body: 'You have 200k tokens left this month.' });
	await page.goto('/inbox', { waitUntil: 'networkidle' });
	await expect(page.getByRole('article', { name: 'Usage at 80%' })).toBeVisible();
	await page.getByRole('banner').getByRole('button', { name: 'Mark all read' }).click();
	await expect(page.getByText('All marked read')).toBeVisible();
	await expect(page.getByRole('article', { name: 'Usage at 80%' })).toBeHidden();
	await expect(page.getByRole('banner').getByRole('button', { name: 'Mark all read' })).toBeDisabled();

	// Updates shows only notifications.
	await page.getByRole('radio', { name: /Updates/ }).click();
	await expect(row(page, 'Nomi speaks Javanese now')).toBeVisible();
});

test('a quick reminder is read from one line, set, and cancelled with undo', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/inbox', { waitUntil: 'networkidle' });
	await expect(page.getByText('Nothing in the next 7 days.')).toBeVisible();

	// "New reminder" puts the person in the line.
	await page.getByRole('banner').getByRole('button', { name: 'New reminder' }).click();
	await expect(quick(page)).toBeFocused();

	// A day with no time: it asks for one.
	await quick(page).fill('water the plants tomorrow');
	await expect(page.getByText('add a time')).toBeVisible();
	await quick(page).press('Enter');
	await expect(page.getByText('Say when, like “at 10am” or “tomorrow at 9”.')).toBeVisible();

	// Every month: the chips say what was read, and it lands in the backend as monthly.
	await quick(page).fill('pay rent every 25th at 10am');
	await expect(page.getByText('monthly', { exact: true })).toBeVisible();
	await expect(page.getByText('10:00', { exact: true })).toBeVisible();
	await quick(page).press('Enter');
	await expect(page.getByText(/^Reminder set for .+ · 10:00$/)).toBeVisible();
	await expect(quick(page)).toHaveValue('');
	await expect.poll(async () => (await reminders(account)).map((one) => [one.title, one.recurrence])).toEqual([['Pay rent', 'monthly']]);

	// In two hours: in Coming up, and under Reminders with its tools.
	await quick(page).fill('stretch in 2 hours');
	await quick(page).press('Enter');
	await expect(page.getByRole('complementary').getByText('Stretch', { exact: true })).toBeVisible();
	await page.getByRole('radio', { name: /Reminders/ }).click();
	await expect(row(page, 'Stretch').getByText(/^in (1h 59m|2h)/)).toBeVisible();
	await expect(row(page, 'Pay rent').getByText('reminder · every month, day 25')).toBeVisible();

	// Cancelled, then undone: it comes back and the backend never heard.
	await row(page, 'Stretch').getByRole('button', { name: 'More for Stretch' }).click();
	await page.getByRole('menuitem', { name: 'Cancel reminder' }).click();
	await expect(row(page, 'Stretch')).toBeHidden();
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(row(page, 'Stretch')).toBeVisible();

	// Cancelled for good: sent after its ten seconds.
	await row(page, 'Stretch').getByRole('button', { name: 'More for Stretch' }).click();
	await page.getByRole('menuitem', { name: 'Cancel reminder' }).click();
	await expect.poll(async () => (await reminders(account)).find((one) => one.title === 'Stretch')?.status, { timeout: 20_000 }).toBe('cancelled');
});

test('a reminder that went off needs the person until it’s done, or snoozed', async ({ page, context }) => {
	const account = await createAccount();
	const due = new Date(Date.now() + 3_600_000).toISOString();
	const made = async (title: string) => (await (await api(account, '/api/reminders', { method: 'POST', body: JSON.stringify({ title, due_at: due }) })).json()) as Reminder;
	const [call, pay] = [await made('Call Siti'), await made('Pay PLN')];
	await fireReminder(call.id);
	await fireReminder(pay.id);
	await signInAs(context, account);
	await page.goto('/inbox', { waitUntil: 'networkidle' });

	const card = page.getByRole('article', { name: 'Call Siti' });
	await expect(card.getByText('reminder · once')).toBeVisible();
	await card.getByRole('button', { name: 'Done' }).click();
	await expect(page.getByText('Marked done')).toBeVisible();
	await expect(card).toBeHidden();

	await page.getByRole('article', { name: 'Pay PLN' }).getByRole('button', { name: 'Snooze 1 hour' }).click();
	await expect(page.getByText('Snoozed for an hour')).toBeVisible();
	await expect(page.getByRole('article', { name: 'Pay PLN' })).toBeHidden();
	await expect(page.getByRole('complementary').getByText('Pay PLN', { exact: true })).toBeVisible();
	expect((await reminders(account)).map((one) => [one.title, one.status]).sort()).toEqual([
		['Call Siti', 'done'],
		['Pay PLN', 'active'],
	]);
});

test('the person chooses which emails they get', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/inbox', { waitUntil: 'networkidle' });
	const promos = page.getByRole('switch', { name: 'Email news and offers' });
	await expect(promos).toHaveAttribute('aria-checked', 'true');
	await promos.click();
	await expect(promos).toHaveAttribute('aria-checked', 'false');
	await expect.poll(async () => (await (await api(account, '/api/notifications/preferences')).json()) as unknown).toEqual({ email_account: true, email_promos: false });
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('switch', { name: 'Email news and offers' })).toHaveAttribute('aria-checked', 'false');
});
