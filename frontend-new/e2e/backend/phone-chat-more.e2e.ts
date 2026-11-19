import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedReply } from './support/seed';

// The rest of a chat on a phone against the real backend (UX contract
// docs/ux/work/phone-chat-more): moving a chat into a project from the phone's own sheet
// (Manage chat & long threads x73ZO, NCfNM).

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

test('a chat moves into a project from the phone’s sheet, and the nav bar names the project', async ({ page, context }) => {
	const account = await createAccount();
	const orgs = (await (await api(account, '/api/orgs')).json()) as { orgs: { id: string }[]; project_org_id: string | null };
	const made = (await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: orgs.project_org_id ?? orgs.orgs[0].id }) })).json()) as { project_id: string };

	await signInAs(context, account);
	await page.goto('/chat/new', { waitUntil: 'networkidle' });
	await page.getByRole('textbox', { name: 'Message' }).fill('Plan this month’s budget');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]{36}$/);
	const id = page.url().split('/').pop()!;
	await page.goto(`/chat/${id}`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('article', { name: 'Nomi' }).getByText('This is a fake response for local development and testing.').first()).toBeVisible();

	// The chat's sheet → Move to a project: the phone's own sheet, not the desktop dialog.
	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Move to a project' }).click();
	const sheet = page.getByRole('dialog', { name: 'Move to a project' });
	await expect(sheet.getByRole('searchbox', { name: 'Search projects' })).toBeVisible();
	const moveButton = sheet.getByRole('button', { name: 'Move chat' });
	await expect(moveButton).toBeDisabled();
	const choice = sheet.getByRole('radio').first();
	const name = (await choice.locator('span.font-mono').textContent())!.trim();
	await choice.click();
	const moved = page.waitForResponse((response) => response.url().endsWith(`/chat/${id}/api/session`) && response.request().method() === 'PATCH');
	await moveButton.click();
	expect((await moved).ok()).toBe(true);
	await expect(page.getByText(`Moved to ${name}`)).toBeVisible();
	// The nav bar names the project under the title (NCfNM).
	await expect(page.getByRole('heading', { level: 1 }).locator('..').getByText(name, { exact: true })).toBeVisible();

	type Sessions = { sessions: { id: string; in_project_id?: string | null }[] };
	const filed = ((await (await api(account, '/api/sessions')).json()) as Sessions).sessions.find((one) => one.id === id);
	expect(filed?.in_project_id).toBe(made.project_id);
});

test('a reply that needs an app opens the phone’s connect sheet; answering yourself leaves the composer to you', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	expect((await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Which weekends am I free in November?' }) })).ok).toBe(true);
	await expect.poll(async () => ((await (await api(account, `/api/sessions/${session_id}/messages`)).json()) as { messages: unknown[] }).messages.length).toBeGreaterThan(1);
	await seedReply(session_id, { content: 'I need your Google Calendar for that.', agent: 'Tara', blocks: [{ kind: 'workspace_connect', reason: 'not_connected', services: ['calendar'], agent_type: 'workspace' }] });

	await page.goto(`/chat/${session_id}`, { waitUntil: 'networkidle' });
	// Crew, memory & connections SYohQ: the sheet opens on its own for the latest reply.
	const sheet = page.getByRole('dialog', { name: 'Connect Google Calendar' });
	await expect(sheet).toBeVisible();
	await expect(sheet.getByText('Nothing is created or changed without asking')).toBeVisible();
	const form = sheet.getByRole('button', { name: 'Connect with Google' }).locator('xpath=ancestor::form');
	await expect(form).toHaveAttribute('action', '/settings/connections?/google');
	await expect(form.locator('input[name="resume"]')).toHaveValue(session_id);

	await sheet.getByRole('button', { name: 'I’ll tell Tara myself' }).click();
	await expect(sheet).not.toBeVisible();
	await expect(page.getByRole('textbox', { name: 'Message' })).toBeFocused();
});

test('rich replies on a phone: the cart, a selection, a link, code, a reminder, a boundary and a table, none wider than the phone', async ({ page, context }) => {
	const account = await createAccount();
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Order this week’s groceries, same as usual' }) });
	await seedReply(session_id, {
		content: 'Your usual Sayurbox list. Ikan kembung is out today; tongkol is the closest swap.',
		agent: 'Nomi',
		blocks: [
			{ kind: 'view', view: 'cart', title: 'Weekly groceries', store: 'Sayurbox', items: [{ name: 'Telur ayam negeri', detail: '1 tray · 10 butir', quantity: 2, price: 29000 }, { name: 'Ikan kembung', detail: 'out of stock', quantity: 0, available: false, swap: 'tongkol' }], delivery: 10000, checkout_url: 'https://sayurbox.com/c/1' },
			{ kind: 'view', view: 'selection', question: 'Track as subscriptions?', items: [{ name: 'Netflix Premium', detail: 'monthly · BCA debit', amount: 186000, selected: true }, { name: 'Spotify Duo', detail: 'monthly · GoPay', amount: 54990 }], confirm_label: 'Track', per: 'month' },
			{ kind: 'view', view: 'link', url: 'https://kosku.id/kos-mawar-tebet', title: 'Kos Mawar Tebet — kamar AC, kamar mandi dalam' },
			{ kind: 'view', view: 'code', code: 'export function weeklyBudget(total: number, weeks = 4) {\n  return Math.round(total / weeks) - 25_000 // leave room for one GoFood order every week\n}', language: 'typescript', file: 'src/lib/budget.ts' },
			{ kind: 'view', view: 'boundary', title: 'I can’t cancel subscriptions', reason: 'Nomi only records money. Instead I can:', alternatives: [{ title: 'Remind me 3 days before the next charge', agent: 'Kala', detail: 'Kala · 2 Nov, 09:00', message: 'Remind me 3 days before Netflix charges' }] },
			{ kind: 'table', variant: 'data', title: 'Food by week', columns: [{ key: 'week', label: 'Week' }, { key: 'groceries', label: 'Groceries' }, { key: 'gofood', label: 'GoFood' }, { key: 'total', label: 'Total' }, { key: 'change', label: 'Change' }], rows: [{ week: '1–7 Oct', groceries: 312000, gofood: 168000, total: 480000, change: '+12%' }] },
		],
	});
	await signInAs(context, account);
	await page.goto(`/chat/${session_id}`, { waitUntil: 'networkidle' });
	const thread = page.getByRole('main');
	await expect(thread.getByText('Weekly groceries')).toBeVisible();
	await expect(thread.getByText('Track as subscriptions?')).toBeVisible();
	await expect(thread.getByText('Kos Mawar Tebet — kamar AC, kamar mandi dalam')).toBeVisible();
	await expect(thread.getByText('src/lib/budget.ts')).toBeVisible();
	await expect(thread.getByText('I can’t cancel subscriptions')).toBeVisible();
	await expect(thread.getByText('Food by week')).toBeVisible();
	// Nothing on the page is wider than the phone: a long code line scrolls inside its card.
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);
	await page.screenshot({ path: test.info().outputPath('rich-replies-phone.png'), fullPage: true });
});
