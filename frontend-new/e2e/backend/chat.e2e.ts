import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';

// Chatting against the real backend and its fake model (Flow / Chat — Ask → think → answer ENb07,
// Message controls Cf6GO, Like & dislike HKYo1, Manage chat & long threads J3ks0, Errors & recovery
// TpuRc): starting a chat from Home, from a starter or a blank chat; the crew's answer coming in
// live; liking it; a turn that fails; renaming, pinning and deleting a chat; the Chats list; and a
// chat that isn't there.

const REPLY = 'This is a fake response for local development and testing.';
/** The fake model fails any turn whose messages hold this. */
const FAIL = '__SIMULATE_TURN_FAILURE__';
const CHAT_URL = /\/chat\/[0-9a-f-]{36}$/;

const json = async <T>(response: Promise<Response>): Promise<T> => (await response).json() as Promise<T>;

/** A chat made through the API, with its first message sent. */
async function chatWith(account: Account, text: string): Promise<string> {
	const { session_id } = await json<{ session_id: string }>(api(account, '/api/sessions', { method: 'POST' }));
	expect((await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text }) })).ok).toBe(true);
	return session_id;
}

/** Names a chat once the crew has named it (the fake model names every chat after its own reply). */
async function named(account: Account, id: string, title: string) {
	type Sessions = { sessions: { id: string; title: string | null }[] };
	await expect.poll(async () => (await json<Sessions>(api(account, '/api/sessions'))).sessions.find((one) => one.id === id)?.title ?? null).not.toBeNull();
	expect((await api(account, `/api/sessions/${id}`, { method: 'PATCH', body: JSON.stringify({ title }) })).ok).toBe(true);
}

/** Opens the app once it has hydrated: before then a click reaches only the server's markup. */
async function open(page: Page, path: string) {
	await page.goto(path, { waitUntil: 'networkidle' });
}

const composer = (page: Page) => page.getByRole('textbox', { name: 'Message' });
const thread = (page: Page) => page.getByRole('main');

test('a message from Home starts a chat, and the answer comes in', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page, '/');

	// Section labels are capitals in CSS only.
	await expect(page.getByText('Try', { exact: true })).toBeVisible();
	await expect(page.getByText('Today', { exact: true })).toBeVisible();
	await expect(page.getByText('All quiet. Nothing new from the crew since you last looked.')).toBeVisible();
	await expect(page.getByText(/^updated \d{2}:\d{2}$/)).toBeVisible();

	await composer(page).fill('Budget belanja Oktober, tolong rangkum');
	await composer(page).press('Enter');

	await expect(page).toHaveURL(CHAT_URL);
	await expect(thread(page).getByText('Budget belanja Oktober, tolong rangkum')).toBeVisible();
	await expect(thread(page).getByText(REPLY)).toBeVisible();
	const id = page.url().split('/').pop()!;
	await expect(page.getByRole('navigation', { name: 'Main' }).locator(`a[href="/chat/${id}"]`)).toBeVisible();
});

test('a starter on Home asks the crew member it belongs to', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page, '/');

	await page.getByRole('button', { name: /Start a small website project/ }).click();
	await expect(page).toHaveURL(CHAT_URL);
	await expect(thread(page).getByText('@Koda Start a small website project', { exact: true })).toBeVisible();
	// A build asked for in a chat: the crew answers by starting a project (not the fake model's line).
	await expect(thread(page).getByRole('article')).toBeVisible();
});

test('a blank chat asks what to pick up, and its first message makes the chat', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page, '/chat/new');

	await expect(page.getByRole('heading', { name: 'What should the crew pick up?' })).toBeVisible();
	await composer(page).fill('Ingatkan aku bayar listrik');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page).toHaveURL(CHAT_URL);
	await expect(thread(page).getByText(REPLY)).toBeVisible();
});

test('in a chat, a follow-up is answered and a reply can be liked', async ({ page, context }) => {
	const account = await createAccount();
	const id = await chatWith(account, 'Halo Nomi');
	await signInAs(context, account);
	await open(page, `/chat/${id}`);

	await expect(thread(page).getByText(REPLY)).toHaveCount(1);
	await composer(page).fill('Satu lagi, ya');
	await composer(page).press('Enter');
	await expect(thread(page).getByText('Satu lagi, ya')).toBeVisible();
	await expect(thread(page).getByText(REPLY)).toHaveCount(2);

	const like = thread(page).getByRole('button', { name: 'Good reply' }).last();
	// Shown at once; saved by the call it makes, which the reload below must not overtake.
	const saved = page.waitForResponse((response) => response.url().endsWith('/feedback') && response.request().method() === 'PUT');
	await like.click();
	await expect(like).toHaveAttribute('aria-pressed', 'true');
	expect((await saved).ok()).toBe(true);
	await open(page, `/chat/${id}`);
	await expect(thread(page).getByRole('button', { name: 'Good reply' }).last()).toHaveAttribute('aria-pressed', 'true');
});

test('a turn that fails says what happened', async ({ page, context }) => {
	const account = await createAccount();
	const id = await chatWith(account, 'Halo');
	await signInAs(context, account);
	await open(page, `/chat/${id}`);
	await expect(thread(page).getByText(REPLY)).toBeVisible();

	await composer(page).fill(`Coba ini ${FAIL}`);
	await composer(page).press('Enter');
	await expect(thread(page).getByText('Couldn’t reply')).toBeVisible();
	await expect(thread(page).getByText(/Something went wrong talking to the AI provider/)).toBeVisible();
});

test('a chat is renamed and pinned from its menu, and deleted with a moment to undo', async ({ page, context }) => {
	const account = await createAccount();
	const id = await chatWith(account, 'Rencana liburan');
	await signInAs(context, account);
	await open(page, `/chat/${id}`);
	await expect(thread(page).getByText(REPLY)).toBeVisible();
	const recent = page.getByRole('navigation', { name: 'Main' }).locator(`a[href="/chat/${id}"]`);

	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('menuitem', { name: 'Rename' }).click();
	const name = page.getByRole('textbox', { name: /name/i });
	await name.fill('Liburan ke Bali');
	await name.press('Enter');
	await expect(page.getByRole('banner').getByText('Liburan ke Bali')).toBeVisible();
	await expect(recent).toContainText('Liburan ke Bali');

	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('menuitem', { name: 'Pin to sidebar' }).click();
	await expect(page.getByText('Pinned to sidebar')).toBeVisible();
	await expect
		.poll(async () => (await json<{ sessions: { id: string; pinned?: boolean; pinned_at?: string | null }[] }>(api(account, '/api/sessions'))).sessions.find((one) => one.id === id))
		.toMatchObject({ id });

	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('menuitem', { name: 'Delete chat' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Delete chat' }).click();
	await expect(page).toHaveURL(/\/$/);
	await expect(page.getByText('Chat deleted')).toBeVisible();
	await expect(recent).toHaveCount(0);
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(page).toHaveURL(new RegExp(`/chat/${id}$`));
	await expect(thread(page).getByText(REPLY)).toBeVisible();

	// Deleted for good: leaving the app sends it at once.
	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('menuitem', { name: 'Delete chat' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Delete chat' }).click();
	await expect(page.getByText('Chat deleted')).toBeVisible();
	await page.goto('about:blank');
	await expect.poll(async () => (await api(account, `/api/sessions/${id}/messages`)).status).toBe(404);
});

test('the Chats page lists chats, pinned first, and finds one', async ({ page, context }) => {
	const account = await createAccount();
	const kopi = await chatWith(account, 'Menu kopi susu');
	const arisan = await chatWith(account, 'Jadwal arisan');
	await named(account, kopi, 'Menu kopi susu');
	await named(account, arisan, 'Jadwal arisan');
	expect((await api(account, `/api/sessions/${kopi}`, { method: 'PATCH', body: JSON.stringify({ pinned: true }) })).ok).toBe(true);
	await signInAs(context, account);
	await open(page, '/chats');

	await expect(page.getByRole('heading', { level: 1, name: 'Chats' })).toBeVisible();
	const list = thread(page);
	await expect(list.getByRole('region', { name: 'Pinned' }).locator(`a[href="/chat/${kopi}"]`)).toBeVisible();
	await expect(list.getByRole('link')).toHaveCount(2);

	await list.getByRole('searchbox', { name: 'Search chats' }).fill('arisan');
	await expect(list.getByRole('link')).toHaveCount(1);
	await list.getByRole('searchbox', { name: 'Search chats' }).fill('tidak ada');
	await expect(list.getByText('No chats match')).toBeVisible();
});

test('a chat that isn’t there, or is someone else’s, says so', async ({ page, context }) => {
	const owner = await createAccount();
	const theirs = await chatWith(owner, 'Rahasia');
	const account = await createAccount();
	await signInAs(context, account);

	for (const id of [theirs, '00000000-0000-4000-8000-000000000000']) {
		const response = await page.goto(`/chat/${id}`);
		expect(response?.status()).toBe(404);
		await expect(page.getByText('This chat isn’t here', { exact: true })).toBeVisible();
		await expect(page.getByRole('complementary', { name: 'Your crew' })).toHaveCount(0);
	}
});
