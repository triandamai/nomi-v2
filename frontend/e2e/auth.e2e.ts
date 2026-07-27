import { expect, test } from '@playwright/test';

function uniqueEmail(): string {
	return `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
}

test('register with a new account lands on the authenticated shell', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/');
	await expect(page.getByText(email)).toBeVisible();
});

test('registering with an already-used email shows an inline error', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.context().clearCookies();
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();

	await expect(page).toHaveURL('/register');
	await expect(page.getByText(/already registered/i)).toBeVisible();
});

test('login with wrong password shows an inline error', async ({ page }) => {
	const email = uniqueEmail();

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.context().clearCookies();
	await page.goto('/login');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('wrong password');
	await page.getByRole('button', { name: 'Log in' }).click();

	await expect(page).toHaveURL('/login');
	await expect(page.getByText(/invalid email or password/i)).toBeVisible();
});
