import { expect, test } from '@playwright/test';

async function registerAndStartChat(page: import('@playwright/test').Page): Promise<string> {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);

	return email;
}

test('sending a message shows the user bubble and the fake assistant reply', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('hello there');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('hello there')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();
});

test('a session history persists across a reload', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('remember this');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText('remember this')).toBeVisible();

	await page.reload();
	await expect(page.getByRole('main').getByText('remember this')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();
});

test('visiting a nonexistent session redirects to the empty state', async ({ page }) => {
	await registerAndStartChat(page);

	await page.goto('/chat/00000000-0000-0000-0000-000000000000');
	await expect(page).toHaveURL('/');
});

test('a backend 502 on send shows an inline "no reply yet" note without losing the sent text', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('__SIMULATE_TURN_FAILURE__');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('__SIMULATE_TURN_FAILURE__')).toBeVisible();
	await expect(page.getByText(/no reply yet/i)).toBeVisible();
});
