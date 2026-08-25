import { expect, test, type Page } from '@playwright/test';
import { promoteToPlatformAdmin } from './support/db';

function uniqueEmail(prefix: string): string {
	return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
}

async function registerViaUi(page: Page, email: string, orgName: string): Promise<void> {
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill(orgName);
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');
}

async function loginViaAdminUi(page: Page, email: string): Promise<void> {
	await page.goto('/admin/login');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByRole('button', { name: 'Log in' }).click();
}

test('visiting the admin area while logged out redirects to admin login', async ({ page }) => {
	await page.goto('/admin');
	await expect(page).toHaveURL('/admin/login');
});

test('a non-admin account is redirected to admin login as forbidden', async ({ page }) => {
	const email = uniqueEmail('nonadmin');
	await registerViaUi(page, email, 'Acme');
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);

	await expect(page).toHaveURL('/admin/login?error=forbidden');
	await expect(page.getByText(/does not have admin access/i)).toBeVisible();
});

test('a platform admin can log in and reach the admin dashboard', async ({ page }) => {
	const email = uniqueEmail('admin');
	await registerViaUi(page, email, 'Acme');
	await promoteToPlatformAdmin(email);
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);

	await expect(page).toHaveURL('/admin');
	await expect(page.getByRole('heading', { name: 'Admin dashboard' })).toBeVisible();
});

test('a platform admin can configure the fake LLM provider', async ({ page }) => {
	const email = uniqueEmail('admin-settings');
	await registerViaUi(page, email, 'Acme');
	await promoteToPlatformAdmin(email);
	await page.context().clearCookies();

	await loginViaAdminUi(page, email);
	await expect(page).toHaveURL('/admin');

	// The LLM settings page was rewritten (user-selectable LLM models plan, Task 7) from a single
	// global provider form into a list of named models with an "Add model" create flow — update
	// this test to match rather than the old single-form UI it originally exercised. The catalog
	// is global (no org scoping), so use a unique label to stay correct across repeated local runs.
	const modelLabel = `Fake model ${Date.now()}-${Math.random().toString(36).slice(2)}`;
	await page.goto('/admin/settings/llm');
	await page.getByRole('button', { name: '+ Add model' }).click();
	await page.getByLabel('Label').fill(modelLabel);
	await page.getByLabel('Provider').selectOption('fake');
	await page.getByRole('button', { name: 'Add model' }).click();

	await expect(page.getByText(modelLabel, { exact: true })).toBeVisible();
	await page.reload();
	await expect(page.getByText(modelLabel, { exact: true })).toBeVisible();
});
