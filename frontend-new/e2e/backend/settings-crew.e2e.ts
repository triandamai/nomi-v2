import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedNomiModel, seedPersonality } from './support/seed';

// Settings › Crew against the real backend (App / Settings / Crew vVUeh): the roster in its groups,
// finding a member, Koda's coding model changed and kept, and the personality's versions.

test('the crew: the roster, a member, and Koda’s model', async ({ page, context }) => {
	const account = await createAccount();
	await seedNomiModel(account.email);
	await signInAs(context, account);
	await page.goto('/settings/connections', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Crew' }).click();
	await expect(page).toHaveURL(/\/settings\/crew$/);
	await expect(page.getByRole('banner').getByText(/^\d+ members/)).toBeVisible();

	const roster = page.getByRole('navigation', { name: 'Crew members' });
	await expect(roster.getByRole('region', { name: 'Core' }).getByRole('link')).toHaveCount(2);
	await expect(roster.getByRole('region', { name: 'Specialists' })).toBeVisible();
	// Nomi is shown first, with the chat model.
	await expect(page.getByRole('heading', { name: 'Nomi', exact: true })).toBeVisible();
	await expect(page.getByRole('combobox', { name: 'Model' })).toBeVisible();

	await roster.getByRole('searchbox').fill('koda');
	await expect(roster.getByRole('link')).toHaveCount(1);
	await roster.getByRole('link', { name: /Koda/ }).click();
	await expect(page).toHaveURL(/\?member=coding$/);
	await expect(page.getByRole('heading', { name: 'Koda', exact: true })).toBeVisible();

	// Koda builds with the chat's model from now on, and that's kept.
	const model = page.getByRole('combobox', { name: 'Model' });
	await model.selectOption('same_as_chat');
	await expect(page.getByText('Model saved')).toBeVisible();
	type Models = { coding: { kind: string } };
	await expect.poll(async () => ((await (await api(account, '/api/llm/models')).json()) as Models).coding.kind).toBe('same_as_chat');
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('combobox', { name: 'Model' })).toHaveValue('same_as_chat');

	// Someone whose model follows the chat.
	await page.goto('/settings/crew?member=money', { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { name: 'Dana', exact: true })).toBeVisible();
	await expect(page.getByText('Works on the model of the chat it’s helping in.')).toBeVisible();
	await expect(page.getByRole('combobox', { name: 'Model' })).toHaveCount(0);

	// Personality: its versions, and an earlier one brought back (as a new version, in use).
	await seedPersonality(account.email, ['Friendly and chatty', 'Formal, full sentences']);
	await page.goto('/settings/crew?member=personality', { waitUntil: 'networkidle' });
	await page.getByRole('tab', { name: 'Versions · 2' }).click();
	await expect(page.getByText('v2 · 2 versions')).toBeVisible();
	await expect(page.getByRole('listitem').filter({ hasText: 'Formal, full sentences' }).getByText('current')).toBeVisible();
	await page.getByRole('button', { name: 'Restore v1' }).click();
	await expect(page.getByText('Personality v1 is back')).toBeVisible();
	await expect(page.getByText('v3 · 3 versions')).toBeVisible();
	await expect(page.getByRole('listitem').filter({ hasText: 'v3' }).getByText('Friendly and chatty')).toBeVisible();
});
