import { expect, test } from '@playwright/test';

test('the empty state shows a greeting and starting a new chat navigates to it', async ({ page }) => {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await expect(page.getByRole('heading', { name: `Hi, ${email}!` })).toBeVisible();

	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);
});
