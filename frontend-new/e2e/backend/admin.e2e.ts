import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAgain, signInAs, type Account } from './support/accounts';
import { makePlatformAdmin } from './support/db';

// The v2 admin console at /admin against the real backend: who may open it (signed out, no admin
// area, a platform admin), its navigation, the overview, the crew at work live, and usage by
// month. A platform admin is made in the database (nothing in the API makes one) and signs in again
// so the token carries it.

/** A platform admin, signed in with a token that says so. */
async function platformAdmin(): Promise<Account> {
	const account = await createAccount({ workspace: 'Admin HQ' });
	await makePlatformAdmin(account.email);
	return signInAgain(account);
}

async function open(page: Page, path: string) {
	await page.goto(path, { waitUntil: 'networkidle' });
}

const nav = (page: Page) => page.getByRole('navigation', { name: 'Admin' });

test('someone signed out signs in first, and comes back to the console', async ({ request }) => {
	const response = await request.get('/admin/usage', { maxRedirects: 0 });
	expect(response.status()).toBe(303);
	expect(response.headers().location).toBe('/sign-in?redirect_to=%2Fadmin%2Fusage');
});

test('someone without an admin area is told so, and nothing of the console shows', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const response = await page.goto('/admin');
	expect(response?.status()).toBe(403);
	await expect(page.getByText('No access to the admin console')).toBeVisible();
	await expect(nav(page)).toHaveCount(0);
	// The backend says the same, whatever the console shows.
	expect((await api(account, '/api/admin/dashboard')).status).toBe(403);
});

test('a platform admin sees every area, and the overview of how Nomi is doing', async ({ page, context }) => {
	const admin = await platformAdmin();
	await signInAs(context, admin);
	await open(page, '/admin');

	for (const area of ['Overview', 'Live agents', 'Usage', 'Users & access', 'Plans', 'Notifications', 'Models', 'Providers', 'Embedding', 'Tools', 'Custom agents', 'Support', 'Maintenance', 'Integrations', 'Configuration']) {
		await expect(nav(page).getByRole('link', { name: area, exact: true })).toBeVisible();
	}
	await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
	await expect(page.getByText('At a glance', { exact: true })).toBeVisible();
	await expect(page.getByText('The overview couldn’t load')).toHaveCount(0);
});

test('the crew at work shows live, and says when nobody is', async ({ page, context }) => {
	const admin = await platformAdmin();
	await signInAs(context, admin);
	await open(page, '/admin/agents');

	await expect(page.getByRole('heading', { level: 1, name: 'Live agents' })).toBeVisible();
	await expect(page.getByText('Live', { exact: true })).toBeVisible();
	await expect(page.getByText('The crew’s work couldn’t load')).toHaveCount(0);
});

test('usage shows a month and moves between months', async ({ page, context }) => {
	const admin = await platformAdmin();
	// Something to count this month: a chat the fake model answers.
	const { session_id } = (await (await api(admin, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	expect((await api(admin, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Halo' }) })).ok).toBe(true);
	await signInAs(context, admin);
	await open(page, '/admin/usage');

	await expect(page.getByRole('heading', { level: 1, name: 'Usage' })).toBeVisible();
	await expect(page.getByText('Usage couldn’t load')).toHaveCount(0);
	await page.getByRole('link', { name: 'Previous month' }).click();
	await expect(page).toHaveURL(/\/admin\/usage\?month=\d{4}-\d{2}$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Usage' })).toBeVisible();
});

test.describe('users & access', () => {
	test('a platform admin finds someone and changes their profile, admin role, workspaces and allowance', async ({ page, context }) => {
		const admin = await platformAdmin();
		const person = await createAccount({ workspace: 'Warung Bu Siti' });
		await signInAs(context, admin);
		await open(page, '/admin/users');

		await page.getByRole('searchbox', { name: 'Search people' }).fill(person.email);
		await page.getByRole('searchbox', { name: 'Search people' }).press('Enter');
		await expect(page).toHaveURL(/\/admin\/users\?query=/);
		const people = page.getByRole('table', { name: 'People' });
		await expect(people.getByRole('row')).toHaveCount(2);
		await people.getByRole('link', { name: person.email }).click();
		await expect(page.getByRole('heading', { level: 1, name: person.email })).toBeVisible();

		// Profile.
		await page.getByLabel('Display name').fill('Siti Rahayu');
		await page.getByRole('region', { name: 'Profile' }).getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Profile saved')).toBeVisible();

		// Admin access: view on Users & access, within the admin's own.
		const role = page.getByRole('group', { name: `Admin access for ${person.email}` });
		await role.getByRole('radiogroup', { name: 'Users & access' }).getByRole('radio', { name: 'View' }).click();
		await page.getByRole('region', { name: 'Admin access' }).getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Admin access saved. It applies from their next sign-in.')).toBeVisible();

		// A workspace: the admin's own, as a member.
		await page.getByLabel('Workspace', { exact: true }).selectOption({ label: 'Admin HQ' });
		await page.getByRole('button', { name: 'Add to workspace' }).click();
		await expect(page.getByText('Added to the workspace')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Remove from Admin HQ' })).toBeVisible();

		// An allowance of their own until the end of the year, with a note.
		await page.getByLabel('Own allowance (tokens a month)').fill('2 000 000');
		await page.getByLabel('Until (YYYY-MM-DD, UTC)').fill('2099-12-31');
		await page.getByLabel('Note').fill('Pilot for the warung');
		await page.getByRole('region', { name: 'Plan & allowance' }).getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Plan saved')).toBeVisible();
		await expect(page.getByText('Pilot for the warung')).toBeVisible();

		// What the backend now holds.
		type Detail = { display_name: string | null; permissions: { scope_type: string; resource: string; actions: string[] }[]; memberships: { org_name: string; role: string }[] };
		const id = page.url().split('/').pop()!;
		const saved = (await (await api(admin, `/api/admin/users/${id}`)).json()) as Detail;
		expect(saved.display_name).toBe('Siti Rahayu');
		expect(saved.permissions.filter((grant) => grant.scope_type === 'admin').map((grant) => grant.resource)).toEqual(['user']);
		expect(saved.memberships).toContainEqual({ org_id: expect.any(String), org_name: 'Admin HQ', role: 'member' });
		const subscription = (await (await api(admin, `/api/admin/users/${id}/subscription`)).json()) as { quota_override: number; override_until: string };
		expect(subscription).toMatchObject({ quota_override: 2_000_000, override_until: '2099-12-31T23:59:59Z' });
	});

	test('a wrong allowance is refused on the page, and nothing is sent', async ({ page, context }) => {
		const admin = await platformAdmin();
		const person = await createAccount();
		await signInAs(context, admin);
		const { users } = (await (await api(admin, `/api/admin/users?query=${encodeURIComponent(person.email)}`)).json()) as { users: { id: string }[] };
		await open(page, `/admin/users/${users[0].id}`);

		await page.getByLabel('Until (YYYY-MM-DD, UTC)').fill('31/12/2099');
		await page.getByRole('region', { name: 'Plan & allowance' }).getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Write the day as YYYY-MM-DD.')).toBeVisible();
	});

	test('someone given view on Users sees only that, and can’t change anyone', async ({ page, context }) => {
		const admin = await platformAdmin();
		const viewer = await createAccount();
		const { users } = (await (await api(admin, `/api/admin/users?query=${encodeURIComponent(viewer.email)}`)).json()) as { users: { id: string }[] };
		expect((await api(admin, `/api/admin/users/${users[0].id}/admin-access`, { method: 'PUT', body: JSON.stringify({ access: [{ resource: 'user', level: 'view' }] }) })).ok).toBe(true);
		await signInAs(context, await signInAgain(viewer));
		await open(page, '/admin');

		await expect(page).toHaveURL(/\/admin\/users$/);
		await expect(nav(page).getByRole('link', { name: 'Users & access', exact: true })).toBeVisible();
		await expect(nav(page).getByRole('link', { name: 'Overview', exact: true })).toHaveCount(0);
		await page.getByRole('table', { name: 'People' }).getByRole('link').first().click();
		await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
		await expect(page.getByRole('button', { name: 'Save' })).toHaveCount(0);
		await expect(page.getByLabel('Display name')).toHaveCount(0);
	});
});
