import { expect, test } from '../support/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedMemory } from './support/seed';

// Search against the real backend (App / Home's top bar, "Search chats, memory, files ⌘K"): the
// person's chats, their memory and their projects' files, found as they type, each opening where
// it lives; ⌘K reaches it from anywhere, and New chat moved to ⌘⇧O.

type Started = { session_id: string };

test('finds a chat, a memory and a project’s file, and opens each where it lives', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const chat = (await (await api(account, '/api/sessions', { method: 'POST', body: JSON.stringify({}) })).json()) as Started;
	expect((await api(account, `/api/sessions/${chat.session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Budget belanja Oktober: beras naik 10%' }) })).ok).toBe(true);
	await seedMemory(account.email, { content: 'Belanja mingguan di pasar Kranggan', kind: 'routine' });
	const org = ((await (await api(account, '/api/orgs')).json()) as { orgs: { id: string }[] }).orgs[0].id;
	const project = ((await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as { project_id: string }).project_id;
	await api(account, `/api/projects/${project}/import/files`, { method: 'POST', body: JSON.stringify({ files: [{ path: 'belanja.html', content: '<h1>Belanja</h1>' }], finish: true }) });

	await page.goto('/', { waitUntil: 'networkidle' });
	const search = page.getByRole('combobox', { name: 'Search chats, memory, files' });
	await search.fill('belanja');
	const found = page.getByRole('listbox');
	await expect(found.getByRole('group', { name: 'Chats' }).getByRole('option')).not.toHaveCount(0);
	await expect(found.getByRole('group', { name: 'Memory' }).getByRole('option', { name: /pasar Kranggan/ })).toBeVisible();
	const file = found.getByRole('group', { name: 'Files' }).getByRole('option', { name: /belanja\.html/ });
	await expect(file).toBeVisible();

	// The file opens in its project's code view, on that file.
	await file.click();
	await expect(page).toHaveURL(new RegExp(`/projects/${project}$`));
	await expect(page.getByRole('radio', { name: 'Code' })).toHaveAttribute('aria-checked', 'true');
	await expect(page.getByText('<h1>Belanja</h1>')).toBeVisible();

	// ⌘K from a page without the field goes Home to it; the keys move through what's found.
	await page.keyboard.press('ControlOrMeta+k');
	await expect(page).toHaveURL(/\/$/);
	await expect(page.getByRole('combobox', { name: 'Search chats, memory, files' })).toBeFocused();
	await page.keyboard.type('Oktober');
	await expect(page.getByRole('group', { name: 'Chats' }).getByRole('option').first()).toBeVisible();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(page).toHaveURL(new RegExp(`/chat/${chat.session_id}$`));
});

test('says when nothing was found, and New chat is ⌘⇧O', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/', { waitUntil: 'networkidle' });
	const search = page.getByRole('combobox', { name: 'Search chats, memory, files' });
	await search.fill('zzqx');
	await expect(page.getByText('Nothing found for “zzqx”')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(page.getByText('Nothing found for “zzqx”')).toHaveCount(0);
	// Off the field, onto something that does nothing when clicked (the middle of Home is a starter).
	await page.getByRole('heading', { level: 1 }).click();
	await page.keyboard.press('ControlOrMeta+Shift+O');
	await expect(page).toHaveURL(/\/chat\/new/);
});
