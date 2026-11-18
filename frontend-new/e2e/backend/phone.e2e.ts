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

test('a chat on a phone: New chat, the nav bar, the bubble, the action bar, and the sheets', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/chat/new', { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: 'What should the crew pick up?' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' })).toHaveCount(0);
	// The pen's own starters (kEp8c): Nomi's is always there.
	await expect(page.getByRole('button', { name: /What can I build in 10 minutes\?/ })).toBeVisible();
	await page.getByRole('textbox', { name: 'Message' }).fill('How much did I spend on food in October?');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]{36}$/);
	const id = page.url().split('/').pop()!;
	const reply = page.getByRole('article', { name: 'Nomi' });
	await expect(reply.getByText('This is a fake response for local development and testing.')).toBeVisible();

	// The round action bar: Like shows at once and is kept (opened again so the finished reply is
	// the one clicked, not the live one it replaces).
	await page.goto(`/chat/${id}`, { waitUntil: 'networkidle' });
	const like = reply.getByRole('button', { name: 'Good reply' });
	const saved = page.waitForResponse((response) => response.url().endsWith('/feedback') && response.request().method() === 'PUT');
	await like.click();
	await expect(like).toHaveAttribute('aria-pressed', 'true');
	expect((await saved).ok()).toBe(true);
	await page.reload({ waitUntil: 'networkidle' });
	await expect(reply.getByRole('button', { name: 'Good reply' })).toHaveAttribute('aria-pressed', 'true');

	// A thread of one day has no date divider (Answer yr0gh).
	await expect(page.getByText('Today', { exact: true })).toHaveCount(0);

	// More: the reply's sheet, then Regenerate's own sheet (each result kept as a version, then the
	// other models the chat can move to), then try again (which asks the backend).
	type Options = { groups: { models: { key: string; enabled: boolean }[] }[]; current: { key: string } | null };
	const options = (await (await api(account, '/api/llm/model-options')).json()) as Options;
	const others = options.groups.flatMap((group) => group.models).filter((model) => model.enabled && model.key !== options.current?.key);
	await reply.getByRole('button', { name: 'More' }).click();
	await expect(page.getByRole('dialog').getByRole('button', { name: /Try another model/ })).toHaveCount(others.length ? 1 : 0);
	await page.getByRole('dialog').getByRole('button', { name: 'Regenerate' }).click();
	await expect(page.getByRole('dialog').getByText('Each result is kept as a version')).toBeVisible();
	await expect(page.getByRole('dialog').getByText('With another model')).toHaveCount(others.length ? 1 : 0);
	const regenerated = page.waitForResponse((response) => response.url().endsWith('/regenerate') && response.request().method() === 'POST');
	await page.getByRole('dialog').getByRole('button', { name: 'Try again' }).click();
	expect((await regenerated).ok()).toBe(true);
	await expect(page.getByRole('dialog')).toHaveCount(0);

	// The nav bar's More is the chat's sheet (how many messages, who answers): rename, then pin.
	await page.getByRole('button', { name: 'Chat options' }).click();
	await expect(page.getByRole('dialog').getByText(/^\d+ messages · Nomi$/)).toBeVisible();
	await page.getByRole('dialog').getByRole('button', { name: 'Rename' }).click();
	await page.getByRole('textbox', { name: 'Chat name' }).fill('Belanja makan Oktober');
	await page.getByRole('button', { name: 'Save name' }).click();
	await expect(page.getByRole('heading', { name: 'Belanja makan Oktober' })).toBeVisible();
	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Pin to top' }).click();
	type Sessions = { sessions: { id: string; title: string | null; pinned: boolean }[] };
	await expect.poll(async () => ((await (await api(account, '/api/sessions')).json()) as Sessions).sessions.find((one) => one.id === id)).toMatchObject({ title: 'Belanja makan Oktober', pinned: true });

	// Back is Chats.
	await page.getByRole('button', { name: 'Back' }).click();
	await expect(page).toHaveURL(/\/chats$/);
});

test('Projects on a phone: the header, New project from its sheet, the card’s sheet, and Sort', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/projects', { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: 'Projects' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Projects' })).toHaveAttribute('aria-current', 'page');
	await expect(page.getByText('No projects yet')).toBeVisible();

	// New: the sheet's composer starts it, and it opens where Koda builds it.
	await page.getByRole('button', { name: 'New', exact: true }).click();
	const sheet = page.getByRole('dialog', { name: 'New project' });
	await sheet.getByRole('textbox', { name: 'Describe an app to build' }).fill('A one-page menu for Warung Kopi');
	await sheet.getByRole('button', { name: 'Start building' }).click();
	await expect(page).toHaveURL(/\/projects\/[0-9a-f-]{36}$/);
	const original = page.url().split('/').pop()!;

	// Back on the list: the phone card, its ⋯ sheet, Duplicate.
	await page.goto('/projects', { waitUntil: 'networkidle' });
	const card = page.getByRole('article').first();
	await expect(card).toBeVisible();
	const name = (await card.getAttribute('aria-label'))!;
	await card.getByRole('button', { name: `${name} actions` }).click();
	const actions = page.getByRole('dialog', { name });
	await expect(actions.getByRole('button', { name: 'Rename' })).toBeVisible();
	await actions.getByRole('button', { name: 'Duplicate' }).click();
	type Projects = { projects: unknown[] };
	await expect.poll(async () => ((await (await api(account, '/api/projects')).json()) as Projects).projects.length).toBe(2);

	// Stop: an action sheet; Keep building leaves it be.
	const stopping = page.getByRole('button', { name: 'Stop' }).first();
	if (await stopping.isVisible()) {
		await stopping.click();
		const ask = page.getByRole('dialog', { name: /Stop building/ });
		await expect(ask.getByRole('button', { name: 'Stop building' })).toBeVisible();
		await ask.getByRole('button', { name: 'Keep building' }).click();
		await expect(ask).toHaveCount(0);
	}

	// Delete the copy: its sheet wants the name typed.
	await page.goto('/projects', { waitUntil: 'networkidle' });
	type Listed = { projects: { id: string; name: string }[] };
	const copyRow = ((await (await api(account, '/api/projects')).json()) as Listed).projects.find((one) => one.id !== original)!;
	const copyName = copyRow.name;
	const copy = page.locator(`[data-project="${copyRow.id}"]`).getByRole('article');
	await copy.getByRole('button', { name: `${copyName} actions` }).click();
	await page.getByRole('dialog', { name: copyName }).getByRole('button', { name: 'Delete project' }).click();
	const remove = page.getByRole('dialog', { name: `Delete ${copyName}?` });
	await expect(remove.getByRole('button', { name: 'Delete project' })).toBeDisabled();
	await remove.getByRole('textbox', { name: `Type ${copyName} to confirm` }).fill(copyName);
	await remove.getByRole('button', { name: 'Delete project' }).click();
	await expect.poll(async () => ((await (await api(account, '/api/projects')).json()) as Projects).projects.length).toBe(1);

	// Sort: a sheet, the chosen one checked.
	await page.getByRole('button', { name: 'Sort' }).click();
	const sort = page.getByRole('dialog', { name: 'Sort projects' });
	await sort.getByRole('button', { name: /Name/ }).click();
	await page.getByRole('button', { name: 'Sort' }).click();
	await expect(page.getByRole('dialog', { name: 'Sort projects' }).getByRole('button', { name: /Name/ })).toHaveAttribute('aria-pressed', 'true');
});

test('a chat’s controls on a phone: your message’s sheet and editing, a reply’s sheet and dislike, share, delete', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/chat/new', { waitUntil: 'networkidle' });
	await page.getByRole('textbox', { name: 'Message' }).fill('How much did I spend on food in October?');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]{36}$/);
	const id = page.url().split('/').pop()!;
	await page.goto(`/chat/${id}`, { waitUntil: 'networkidle' });
	const reply = page.getByRole('article', { name: 'Nomi' });
	await expect(reply.getByText('This is a fake response for local development and testing.')).toBeVisible();

	// Long-pressing your message: its sheet, then editing it in place.
	const mine = page.getByText('How much did I spend on food in October?');
	const at = (await mine.boundingBox())!;
	await page.mouse.move(at.x + 20, at.y + at.height / 2);
	await page.mouse.down();
	await page.waitForTimeout(650);
	await page.mouse.up();
	const yours = page.getByRole('dialog', { name: 'Your message' });
	await expect(yours.getByRole('button')).toHaveText(['', 'Edit and resend', 'Copy text', 'Select text']);
	// Opened by a long press, the sheet has the focus, not its close (no ring on Close: rQ4w3).
	await expect(yours.getByRole('button', { name: 'Close' })).not.toBeFocused();
	await yours.getByRole('button', { name: 'Edit and resend' }).click();
	const editor = page.getByRole('textbox', { name: 'Your message' });
	await expect(editor).toBeFocused();
	await expect(page.getByText('saves as version 2')).toBeVisible();
	await editor.fill('How much did I spend on food in October, without office lunches?');
	await page.getByRole('button', { name: 'Send' }).first().click();
	await expect(page.getByText('How much did I spend on food in October, without office lunches?')).toBeVisible();

	// Select text selects your message where it stands (once the edited turn has settled).
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('article', { name: 'Nomi' }).last().getByRole('button', { name: 'More' })).toBeVisible();
	const edited = page.getByText('How much did I spend on food in October, without office lunches?');
	const spot = (await edited.boundingBox())!;
	await page.mouse.move(spot.x + 20, spot.y + spot.height / 2);
	await page.mouse.down();
	await page.waitForTimeout(650);
	await page.mouse.up();
	await page.getByRole('dialog', { name: 'Your message' }).getByRole('button', { name: 'Select text' }).click();
	await expect.poll(() => page.evaluate(() => getSelection()?.toString())).toBe('How much did I spend on food in October, without office lunches?');

	// A reply's sheet: Bad opens the dislike sheet, sent with a reason.
	await page.getByRole('article', { name: 'Nomi' }).last().getByRole('button', { name: 'More' }).click();
	const sheet = page.getByRole('dialog', { name: 'Reply' });
	await expect(sheet.getByRole('button', { name: 'Good' })).toBeVisible();
	await sheet.getByRole('button', { name: 'Bad' }).click();
	const why = page.getByRole('dialog', { name: 'What went wrong?' });
	await expect(why.getByRole('group', { name: 'What went wrong?' }).getByRole('button')).toHaveText(['Not accurate', 'Wrong crew member', "Didn't do what I asked", 'Too long', 'Out of date', 'Other']);
	await why.getByRole('button', { name: 'Not accurate' }).click();
	await why.getByRole('button', { name: 'Send feedback' }).click();
	await expect(page.getByText('Thanks. The crew learns from this.')).toBeVisible();

	// Share: the phone's sheet.
	await page.getByRole('article', { name: 'Nomi' }).last().getByRole('button', { name: 'Share' }).click();
	await expect(page.getByRole('dialog', { name: 'Share' }).getByRole('button', { name: 'Copy link' })).toBeVisible();
	await expect(page.getByRole('dialog', { name: 'Share' }).getByText('Read-only, no sign-in', { exact: true })).toBeVisible();
	await page.keyboard.press('Escape');

	// Delete asks in an action sheet; Cancel keeps the chat.
	await page.getByRole('button', { name: 'Chat options' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Delete chat' }).click();
	const ask = page.getByRole('dialog', { name: 'Delete this chat?' });
	await expect(ask.getByText(/^\d+ messages and their cards are removed\. Reminders, money records and sent emails stay\.$/)).toBeVisible();
	await expect(ask.getByRole('button', { name: 'Delete chat' })).toBeVisible();
	await ask.getByRole('button', { name: 'Cancel' }).click();
	await expect(page).toHaveURL(new RegExp(`/chat/${id}$`));
});
