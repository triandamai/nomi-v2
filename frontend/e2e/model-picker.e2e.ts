import { expect, test } from '@playwright/test';
import { promoteToPlatformAdmin } from './support/db';

async function registerAdminAndLogin(page: import('@playwright/test').Page): Promise<string> {
	const email = `admin-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;
	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');
	return email;
}

async function startChat(page: import('@playwright/test').Page): Promise<void> {
	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);
}

test('an admin-created model appears in the picker and can be selected', async ({ page }) => {
	const email = await registerAdminAndLogin(page);

	// Registering an org owner does NOT grant platform-admin access — /admin/(protected) requires
	// is_platform_admin in the DB (see backend/src/auth/permissions.rs), which normal registration
	// never sets. Promote directly and re-authenticate through the admin login form, the same
	// pattern admin.e2e.ts already uses, or the admin panel below redirects to
	// /admin/login?error=forbidden and none of its markup ever renders.
	await promoteToPlatformAdmin(email);
	await page.context().clearCookies();
	await page.goto('/admin/login');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByRole('button', { name: 'Log in' }).click();
	await expect(page).toHaveURL('/admin');

	// The admin model catalog is global (not scoped per org — see backend/src/routes/llm_models.rs,
	// no org_id anywhere), so it accumulates across every past run of this suite against the same
	// dev database. A fixed label like "Second Model" collides with leftovers from a prior run and
	// makes `getByRole('button', { name: ... })` ambiguous (strict-mode violation); a per-run unique
	// label with exact matching keeps the test correct regardless of catalog history.
	const modelLabel = `Second Model ${Date.now()}-${Math.random().toString(36).slice(2)}`;

	await page.goto('/admin/settings/llm');
	await page.getByRole('button', { name: '+ Add model' }).click();
	await page.getByLabel('Label').fill(modelLabel);
	await page.getByLabel('Provider').selectOption('fake');
	await page.getByRole('button', { name: 'Add model' }).click();
	await expect(page.getByText(modelLabel, { exact: true })).toBeVisible();

	await page.goto('/');
	await startChat(page);

	await page.getByRole('button', { name: 'Default model' }).click();
	await page.getByRole('button', { name: modelLabel, exact: true }).click();
	await expect(page.getByRole('button', { name: modelLabel, exact: true })).toBeVisible();
});

test('a user can bring their own key using the fake provider and see it become active', async ({ page }) => {
	await registerAdminAndLogin(page);
	await startChat(page);

	await page.getByRole('button', { name: 'Default model' }).click();
	await page.getByRole('button', { name: '+ Use your own API key' }).click();
	await page.getByPlaceholder('Label').fill('My fake key');
	// The <select>'s first option is "anthropic", not "fake" — must select it explicitly, or
	// the save is rejected with "api_key is required for a non-fake provider" since the API
	// key field below is intentionally left blank (fake needs neither model_id nor a key).
	const providerSelects = page.locator('form[action="?/selectCustomModel"] select[name="provider"]');
	await providerSelects.selectOption('fake');
	await page.getByPlaceholder('Model ID').fill('');
	await page.getByRole('button', { name: 'Save & validate' }).click();

	await expect(page.getByRole('button', { name: 'My fake key' })).toBeVisible();
});
