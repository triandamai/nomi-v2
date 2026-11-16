import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// The phone's own screens against the real backend (Flow / Mobile Chat — Start & answer): the tab
// bar in place of the drawer, Chats with its large title, rows, sheet and floating New chat.

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

/** A chat with a message in it, named once the crew has named it (the fake model names each chat). */
async function startChat(account: Parameters<typeof api>[0], text: string): Promise<string> {
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	expect((await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text }) })).ok).toBe(true);
	type Sessions = { sessions: { id: string; title: string | null }[] };
	await expect
		.poll(async () => ((await (await api(account, '/api/sessions')).json()) as Sessions).sessions.find((one) => one.id === session_id)?.title ?? null)
		.not.toBeNull();
	expect((await api(account, `/api/sessions/${session_id}`, { method: 'PATCH', body: JSON.stringify({ title: text }) })).ok).toBe(true);
	return session_id;
}

test('a phone has the tab bar, and Chats as the pen draws it: large title, rows, sheet, New chat', async ({ page, context }) => {
	const account = await createAccount();
	const kopi = await startChat(account, 'Menu kopi susu');
	await startChat(account, 'Jadwal arisan');
	await signInAs(context, account);

	await page.goto('/chats', { waitUntil: 'networkidle' });
	const tabs = page.getByRole('navigation', { name: 'Main' });
	await expect(tabs.getByRole('link', { name: 'Chats' })).toHaveAttribute('aria-current', 'page');
	await expect(tabs.getByRole('link')).toHaveText(['Home', 'Chats', 'Projects', 'Inbox', 'You']);
	await expect(page.getByRole('button', { name: 'Open navigation' })).toHaveCount(0);
	await expect(page.getByRole('heading', { level: 1, name: 'Chats' })).toBeVisible();
	await expect(page.getByRole('link', { name: /Menu kopi susu/ })).toBeVisible();

	// The search narrows the list.
	await page.getByRole('searchbox', { name: 'Search chats' }).fill('arisan');
	await expect(page.getByRole('link', { name: /Menu kopi susu/ })).toHaveCount(0);
	await page.getByRole('searchbox', { name: 'Search chats' }).fill('');

	// A long press opens the chat's sheet; Pin to top pins it.
	const row = page.getByRole('link', { name: /Menu kopi susu/ });
	const box = (await row.boundingBox())!;
	await page.mouse.move(box.x + 60, box.y + box.height / 2);
	await page.mouse.down();
	await page.waitForTimeout(650);
	await page.mouse.up();
	const sheet = page.getByRole('dialog', { name: 'Menu kopi susu' });
	await expect(sheet).toBeVisible();
	await expect(page).toHaveURL(/\/chats$/);
	await sheet.getByRole('button', { name: 'Pin to top' }).click();
	await expect(sheet).toHaveCount(0);
	type Sessions = { sessions: { id: string; pinned: boolean }[] };
	await expect.poll(async () => ((await (await api(account, '/api/sessions')).json()) as Sessions).sessions.find((one) => one.id === kopi)?.pinned).toBe(true);

	// New chat floats above the tab bar.
	await page.getByRole('link', { name: 'New chat' }).click();
	await expect(page).toHaveURL(/\/chat\/new$/);

	// The other tabs go where they say; You is Settings.
	await page.goto('/inbox', { waitUntil: 'networkidle' });
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Inbox' })).toHaveAttribute('aria-current', 'page');
	await page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'You' }).click();
	await expect(page).toHaveURL(/\/settings\/profile$/);
});

test.describe('on a desktop', () => {
	test.use({ viewport: { width: 1512, height: 862 }, userAgent: undefined, hasTouch: false });
	test('there is no tab bar, and Chats is the desktop list', async ({ page, context }) => {
		const account = await createAccount();
		await signInAs(context, account);
		await page.goto('/chats', { waitUntil: 'networkidle' });
		await expect(page.getByRole('link', { name: 'You' })).toHaveCount(0);
		await expect(page.getByRole('heading', { level: 1, name: 'Chats' })).toBeVisible();
		await expect(page.getByText('Every conversation with the crew, pinned ones first.')).toBeVisible();
	});
});
