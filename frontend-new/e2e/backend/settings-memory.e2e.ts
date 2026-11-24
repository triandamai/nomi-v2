import { readFile } from 'node:fs/promises';
import { expect, test } from '../support/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedMemory } from './support/seed';

// Settings › Memory & context against the real backend (App / Settings / Memory hANTE): the
// topics and their strength, the Core and Fading views and a search, correcting and forgetting a
// memory, the export, and telling Nomi something to remember.

type Memory = { id: string; content: string };
const memories = async (account: Parameters<typeof api>[0]) => ((await (await api(account, '/api/memory')).json()) as { memories: Memory[] }).memories;

test('what the crew remembers: read, narrowed, corrected, forgotten, exported', async ({ page, context }) => {
	const account = await createAccount();
	await seedMemory(account.email, { content: 'Short, direct answers', kind: 'preference', weight: 4 });
	await seedMemory(account.email, { content: 'Siti is my sister in Bandung', kind: 'person', weight: 0.3 });
	await seedMemory(account.email, { content: 'Gym Mon, Wed, Fri at 06:30', kind: 'routine' });
	await signInAs(context, account);
	await page.goto('/settings/models', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Memory & context' }).click();
	await expect(page).toHaveURL(/\/settings\/memory$/);
	await expect(page.getByRole('banner').getByText('3 memories · 3 topics')).toBeVisible();
	await expect(page.getByRole('button', { name: /Preferences/ })).toBeVisible();
	await expect(page.getByRole('img', { name: 'core' }).first()).toBeVisible();

	// Narrowed: core, fading, a search.
	await page.getByRole('radio', { name: /Core/ }).click();
	await expect(page.getByRole('button', { name: /People/ })).toBeHidden();
	await page.getByRole('radio', { name: /Fading/ }).click();
	await expect(page.getByText('Siti is my sister in Bandung', { exact: true }).first()).toBeVisible();
	await page.getByRole('radio', { name: /All/ }).click();
	await page.getByRole('searchbox', { name: 'Search memory' }).fill('gym');
	await expect(page.getByText('Gym Mon, Wed, Fri at 06:30', { exact: true }).first()).toBeVisible();
	await expect(page.getByRole('button', { name: /Preferences/ })).toBeHidden();
	await page.getByRole('searchbox', { name: 'Search memory' }).fill('');

	// Corrected in place.
	await page.getByRole('button', { name: /Preferences/ }).click();
	await page.getByRole('button', { name: 'Correct it' }).first().click();
	await page.getByRole('textbox', { name: 'What Nomi should remember' }).fill('Short answers in Bahasa');
	await page.keyboard.press('Enter');
	await expect(page.getByText('Corrected')).toBeVisible();
	await expect.poll(async () => (await memories(account)).map((one) => one.content)).toContain('Short answers in Bahasa');

	// Forgotten.
	await page.getByRole('button', { name: /People/ }).click();
	const siti = page.getByRole('listitem').filter({ hasText: 'Siti is my sister in Bandung' }).last();
	await siti.getByRole('button', { name: 'Forget it' }).click();
	await expect(page.getByText('Forgotten')).toBeVisible();
	await expect.poll(async () => (await memories(account)).map((one) => one.content)).not.toContain('Siti is my sister in Bandung');

	// Exported.
	const download = page.waitForEvent('download');
	await page.getByRole('banner').getByRole('link', { name: 'Export' }).click();
	const file = await (await download).path();
	const exported = JSON.parse(await readFile(file!, 'utf8')) as { memories: Memory[] };
	expect(exported.memories.map((one) => one.content).sort()).toEqual(['Gym Mon, Wed, Fri at 06:30', 'Short answers in Bahasa']);
});

test('telling Nomi something to remember starts a chat with it', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/memory', { waitUntil: 'networkidle' });
	await expect(page.getByText('Nothing remembered yet')).toBeVisible();
	await expect(async () => {
		await page.getByRole('textbox', { name: 'Message' }).fill('I moved to Bandung');
		await expect(page.getByRole('button', { name: 'Send' })).toBeEnabled({ timeout: 1_000 });
	}).toPass();
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]{36}$/);
	await expect(page.getByText('I moved to Bandung').first()).toBeVisible();
});


test('who learned what, the profile, and how memory learns and how much it carries', async ({ page, context }) => {
	const account = await createAccount();
	await seedMemory(account.email, { content: 'Budgets in Rupiah', kind: 'preference', learnedBy: 'money' });
	await seedMemory(account.email, { content: 'Pays Netflix monthly', kind: 'preference', learnedBy: 'money' });
	await seedMemory(account.email, { content: 'Gym at 06:30', kind: 'routine', learnedBy: 'health' });
	await signInAs(context, account);
	await page.setViewportSize({ width: 1512, height: 862 });
	await page.goto('/settings/memory', { waitUntil: 'networkidle' });

	// Learned by: Dana for Preferences, Raga for Routines; each memory says so too.
	await expect(page.getByRole('button', { name: /Preferences.*Dana/ })).toBeVisible();
	await expect(page.getByRole('button', { name: /Routines.*Raga/ })).toBeVisible();
	await page.getByRole('button', { name: /Routines/ }).click();
	await expect(page.getByText(/learned by Raga/)).toBeVisible();

	// The profile: written, kept, cleared and brought back.
	const profile = async () => (await (await api(account, '/api/memory/profile')).json()) as { text: string; tokens: number };
	await page.getByRole('button', { name: 'Edit the profile' }).click();
	await page.getByRole('textbox', { name: 'Your profile' }).fill('Ana, Jakarta. Short answers, in Bahasa.');
	await page.getByRole('button', { name: 'Save' }).click();
	await expect(page.getByText('Profile saved. The crew reads it from the next reply.')).toBeVisible();
	await expect.poll(async () => (await profile()).text).toBe('Ana, Jakarta. Short answers, in Bahasa.');
	await expect(page.getByText('10 / 1,000 tok')).toBeVisible();
	await page.getByRole('button', { name: 'Clear the profile' }).click();
	await expect.poll(async () => (await profile()).text).toBe('');
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect.poll(async () => (await profile()).text).toBe('Ana, Jakarta. Short answers, in Bahasa.');

	// Learning and Budget, in the column beside the topics.
	const settings = async () => (await (await api(account, '/api/memory/settings')).json()) as { search_past_chats: boolean; memory_budget: number; profile_budget: number };
	const column = page.getByRole('complementary', { name: 'Memory settings' });
	await column.getByRole('switch', { name: 'Search past chats' }).click();
	await expect.poll(async () => (await settings()).search_past_chats).toBe(false);
	await column.getByRole('textbox', { name: 'Profile' }).fill('50');
	await column.getByRole('textbox', { name: 'Profile' }).press('Enter');
	await expect(column.getByText('Between 100 and 4,000 tokens.')).toBeVisible();
	await column.getByRole('textbox', { name: 'Memory' }).fill('2200');
	await column.getByRole('textbox', { name: 'Memory' }).press('Enter');
	await expect.poll(async () => (await settings()).memory_budget).toBe(2200);
	expect((await settings()).profile_budget).toBe(1000);
	await page.reload({ waitUntil: 'networkidle' });
	await expect(column.getByRole('switch', { name: 'Search past chats' })).not.toBeChecked();
	await expect(column.getByRole('textbox', { name: 'Memory' })).toHaveValue('2,200');
});
