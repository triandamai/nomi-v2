import { expect, test } from '@playwright/test';
import { api, createAccount, signInAgain, signInAs } from './support/accounts';
import { makePlatformAdmin } from './support/db';

// Maintenance from the admin console against the real backend. It holds everyone on the
// maintenance page, so it runs on its own after every other test (its own project in
// playwright.backend.config.ts) and always turns maintenance off again.

test('an admin holds everyone on the maintenance page, and lets them back', async ({ browser }) => {
	const admin = await createAccount({ workspace: 'Admin HQ' });
	await makePlatformAdmin(admin.email);
	const signedIn = await signInAgain(admin);
	const person = await createAccount();

	const adminContext = await browser.newContext();
	const personContext = await browser.newContext();
	await signInAs(adminContext, signedIn);
	await signInAs(personContext, person);
	const console = await adminContext.newPage();
	const app = await personContext.newPage();

	try {
		await console.goto('/admin/maintenance', { waitUntil: 'networkidle' });
		await console.getByRole('switch', { name: 'Hold everyone on the maintenance page' }).click();
		await console.getByLabel('Message').fill('Moving to a bigger database.');
		await console.getByLabel('Back by: day (YYYY-MM-DD, UTC)').fill('2099-01-01');
		await console.getByLabel('Time (HH:MM, UTC)').fill('03:30');
		await console.getByRole('button', { name: 'Save' }).click();
		await console.getByRole('dialog').getByRole('button', { name: 'Turn on maintenance' }).click();
		await expect(console.getByText('Maintenance is on. Everyone else sees the maintenance page.')).toBeVisible();
		// The console itself stays open.
		await expect(console.getByRole('heading', { level: 1, name: 'Maintenance' })).toBeVisible();

		await app.goto('/');
		await expect(app).toHaveURL(/\/maintenance$/);
		await expect(app.getByText('Moving to a bigger database.')).toBeVisible();

		await console.getByRole('switch', { name: 'Hold everyone on the maintenance page' }).click();
		await console.getByRole('button', { name: 'Save' }).click();
		await expect(console.getByText('Maintenance is off. Nomi is open.')).toBeVisible();
		await app.goto('/');
		await expect(app).not.toHaveURL(/\/maintenance$/);
	} finally {
		// Never leave the shared backend down.
		await api(signedIn, '/api/admin/maintenance', { method: 'PUT', body: JSON.stringify({ enabled: false, message: '', until: null }) });
		await adminContext.close();
		await personContext.close();
	}
});
