import { expect, test } from '@playwright/test';

test('visiting the home page while logged out redirects to login', async ({ page }) => {
	await page.goto('/');
	await expect(page).toHaveURL('/login');
});

test('after logging in, the sidebar and logout are visible; logout returns to login', async ({ page }) => {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByRole('button', { name: '+ New Chat' })).toBeVisible();
	await expect(page.getByText(email)).toBeVisible();

	await page.getByRole('button', { name: 'Log out' }).click();
	await expect(page).toHaveURL('/login');

	await page.goto('/');
	await expect(page).toHaveURL('/login');
});
