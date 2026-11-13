import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAgain, signInAs, type Account } from './support/accounts';
import { deleteAdminModel, deleteConnection, deleteDynamicAgent, makePlatformAdmin, seedSharedEndpoint } from './support/db';
import { APP_URL } from './support/env';

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

/** A plan only this test uses, so tests side by side never share one. */
const uniquePlan = () => {
	const tag = Math.random().toString(36).slice(2, 8);
	return { name: `E2E ${tag}`, slug: `e2e-${tag}` };
};

test.describe('plans', () => {
	test('a plan is added, hidden, and deleted while nobody is on it', async ({ page, context }) => {
		const admin = await platformAdmin();
		const plan = uniquePlan();
		await signInAs(context, admin);
		await open(page, '/admin/plans');

		await page.getByRole('link', { name: 'Add a plan' }).click();
		await expect(page.getByRole('heading', { level: 1, name: 'New plan' })).toBeVisible();
		await page.getByLabel('Name', { exact: true }).fill(plan.name);
		await expect(page.getByLabel('Slug')).toHaveValue(plan.slug);
		await page.getByLabel('Tokens a month').fill('3 000 000');
		await page.getByLabel('Price label').fill('Rp 49.000 / month');
		await page.getByLabel('What it includes').fill('Everything in Free\nPriority answers');
		await page.getByRole('button', { name: 'Create plan' }).click();
		await expect(page.getByRole('heading', { level: 1, name: plan.name })).toBeVisible();
		await expect(page).toHaveURL(/\/admin\/plans\/[0-9a-f-]{36}$/);

		await page.getByRole('switch', { name: 'Offered to people' }).click();
		await page.getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Plan saved')).toBeVisible();
		type Listed = { slug: string; is_active: boolean; monthly_tokens: number; features: string[] };
		const listed = ((await (await api(admin, '/api/admin/plans')).json()) as Listed[]).find((one) => one.slug === plan.slug);
		expect(listed).toMatchObject({ is_active: false, monthly_tokens: 3_000_000, features: ['Everything in Free', 'Priority answers'] });

		await page.getByRole('button', { name: 'Delete plan' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Delete plan' }).click();
		await expect(page).toHaveURL(/\/admin\/plans$/);
		await expect(page.getByRole('link', { name: plan.name })).toHaveCount(0);
	});

	test('a plan someone is on can’t be deleted, and the page says why', async ({ page, context }) => {
		const admin = await platformAdmin();
		const plan = uniquePlan();
		const created = (await (await api(admin, '/api/admin/plans', { method: 'POST', body: JSON.stringify({ ...plan, monthly_tokens: 1000 }) })).json()) as { id: string };
		const person = await createAccount();
		const { users } = (await (await api(admin, `/api/admin/users?query=${encodeURIComponent(person.email)}`)).json()) as { users: { id: string }[] };
		expect((await api(admin, `/api/admin/users/${users[0].id}/subscription`, { method: 'PUT', body: JSON.stringify({ plan_id: created.id }) })).ok).toBe(true);
		await signInAs(context, admin);
		await open(page, `/admin/plans/${created.id}`);

		await page.getByRole('button', { name: 'Delete plan' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Delete plan' }).click();
		await expect(page.getByText('The plan couldn’t be deleted')).toBeVisible();
		await expect(page.getByText('people are or were on this plan; hide it instead')).toBeVisible();
	});
});

test.describe('notifications', () => {
	test('a notification goes to the people on one plan, after a confirm, and shows as sent', async ({ page, context }) => {
		const admin = await platformAdmin();
		const plan = uniquePlan();
		const created = (await (await api(admin, '/api/admin/plans', { method: 'POST', body: JSON.stringify({ ...plan, monthly_tokens: 1000 }) })).json()) as { id: string };
		const person = await createAccount();
		const { users } = (await (await api(admin, `/api/admin/users?query=${encodeURIComponent(person.email)}`)).json()) as { users: { id: string }[] };
		expect((await api(admin, `/api/admin/users/${users[0].id}/subscription`, { method: 'PUT', body: JSON.stringify({ plan_id: created.id }) })).ok).toBe(true);
		await signInAs(context, admin);
		await open(page, '/admin/notifications');

		const title = `Kala reads reminders aloud ${plan.slug}`;
		await page.getByLabel('Title').fill(title);
		await page.getByLabel('Message').fill('Turn it on in Settings › Voice.');
		await page.getByLabel('Link (optional)').fill('/settings');
		await page.getByLabel('Send to').selectOption({ label: `People on ${plan.name}` });
		await page.getByRole('button', { name: 'Send' }).click();
		await expect(page.getByRole('dialog')).toContainText(`Send to People on ${plan.name}?`);
		await page.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
		await expect(page.getByText('Sent to 1 person')).toBeVisible();
		await expect(page.getByRole('table', { name: 'Sent' }).getByText(title)).toBeVisible();

		const inbox = (await (await api(person, '/api/notifications')).json()) as { items: { title: string; link: string | null }[] };
		expect(inbox.items).toContainEqual(expect.objectContaining({ title, link: '/settings' }));
	});

	test('a link outside the app is refused on the page', async ({ page, context }) => {
		const admin = await platformAdmin();
		await signInAs(context, admin);
		await open(page, '/admin/notifications');
		await page.getByLabel('Title').fill('Read this');
		await page.getByLabel('Message').fill('Somewhere else');
		await page.getByLabel('Link (optional)').fill('https://example.com');
		await page.getByRole('button', { name: 'Send' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
		await expect(page.getByText('The link is a page in Nomi, starting with /.')).toBeVisible();
	});
});

test.describe('models, providers and embedding', () => {
	type Model = { id: string; label: string; model_id: string; enabled: boolean; input_usd_per_mtok: number | null; media_inputs: string[] | null; connection_id: string | null };
	const models = async (admin: Account) => (await (await api(admin, '/api/admin/settings/llm/models')).json()) as Model[];
	/**
	 * The backend makes the first model the default for everyone's chats. A default on the fake
	 * provider is made first, so a model a test adds (on an endpoint that doesn't answer) never
	 * becomes the one other tests' chats run on.
	 */
	async function fakeDefault(admin: Account) {
		const listed = (await models(admin)) as (Model & { is_default: boolean })[];
		if (listed.some((one) => one.is_default)) return;
		const made = await api(admin, '/api/admin/settings/llm/models', { method: 'POST', body: JSON.stringify({ label: 'E2E default (fake)', provider: 'fake', model_id: 'fake', api_key: '' }) });
		expect(made.ok).toBe(true);
	}

	test('a model with its own key is added, switched off and deleted', async ({ page, context }) => {
		const admin = await platformAdmin();
		const tag = Math.random().toString(36).slice(2, 8);
		const label = `E2E model ${tag}`;
		await fakeDefault(admin);
		await signInAs(context, admin);
		await open(page, '/admin/models/new');
		try {
			await page.getByLabel('Name people see').fill(label);
			await page.getByRole('radio', { name: 'Its own key' }).click();
			await page.getByLabel('Provider', { exact: true }).selectOption('fake');
			await page.getByLabel('API key').fill('sk-e2e-not-real');
			await page.getByLabel('Model id').fill(`e2e-${tag}`);
			await page.getByLabel('Input', { exact: true }).fill('0.5');
			await page.getByLabel('Output', { exact: true }).fill('1.5');
			await page.getByRole('radio', { name: 'These' }).click();
			await page.getByRole('checkbox', { name: 'Images' }).click();
			await page.getByRole('button', { name: 'Add model' }).click();
			await expect(page).toHaveURL(/\/admin\/models$/);
			const row = page.getByRole('row', { name: new RegExp(label) });
			await expect(row).toBeVisible();
			const added = (await models(admin)).find((one) => one.label === label)!;
			expect(added).toMatchObject({ model_id: `e2e-${tag}`, input_usd_per_mtok: 0.5, media_inputs: ['image'], connection_id: null, enabled: true });

			await row.getByRole('button', { name: `Turn off ${label}` }).click();
			await expect(row.getByRole('button', { name: `Turn on ${label}` })).toBeVisible();
			expect((await models(admin)).find((one) => one.id === added.id)?.enabled).toBe(false);

			await row.getByRole('link', { name: label }).click();
			await page.getByRole('button', { name: 'Delete model' }).click();
			await page.getByRole('dialog').getByRole('button', { name: 'Delete model' }).click();
			await expect(page).toHaveURL(/\/admin\/models$/);
			expect((await models(admin)).some((one) => one.id === added.id)).toBe(false);
		} finally {
			const left = (await models(admin)).find((one) => one.label === label);
			if (left) await deleteAdminModel(left.id);
		}
	});

	test('a wrong price is refused on the page', async ({ page, context }) => {
		const admin = await platformAdmin();
		await signInAs(context, admin);
		await open(page, '/admin/models/new');
		await page.getByLabel('Name people see').fill('Cheap');
		await page.getByRole('radio', { name: 'Its own key' }).click();
		await page.getByLabel('Model id').fill('cheap-1');
		await page.getByLabel('Input', { exact: true }).fill('free');
		await page.getByRole('button', { name: 'Add model' }).click();
		await expect(page.getByText('Prices are dollars per million tokens, zero or more.')).toBeVisible();
	});

	test('connecting a provider says what the backend refused', async ({ page, context }) => {
		const admin = await platformAdmin();
		await signInAs(context, admin);
		await open(page, '/admin/models/providers');
		const connect = page.getByRole('region', { name: 'Connect a provider' });
		await connect.getByLabel('Provider', { exact: true }).selectOption('custom');
		await connect.getByLabel('Name').fill('Plain http');
		await connect.getByLabel('Address').fill('http://llm.e2e.invalid/v1');
		await connect.getByRole('button', { name: 'Connect', exact: true }).click();
		await expect(page.getByText('The provider couldn’t be connected')).toBeVisible();
		await expect(page.getByText('use an https:// address')).toBeVisible();
	});

	test('a connected provider is renamed, runs a model, can’t go while it does, then is disconnected', async ({ page, context }) => {
		const admin = await platformAdmin();
		const tag = Math.random().toString(36).slice(2, 8);
		const label = `E2E endpoint ${tag}`;
		const renamed = `E2E lab ${tag}`;
		await fakeDefault(admin);
		const connection = await seedSharedEndpoint(label);
		await signInAs(context, admin);
		try {
			await open(page, '/admin/models/providers');
			const section = page.getByRole('region', { name: label });
			await expect(section.getByText('No model uses it yet.')).toBeVisible();
			await section.getByLabel('Name').fill(renamed);
			await section.getByRole('button', { name: 'Save' }).click();
			await expect(page.getByRole('region', { name: renamed })).toBeVisible();

			// A model on it, made in the console.
			await open(page, '/admin/models/new');
			await page.getByLabel('Name people see').fill(`${renamed} model`);
			await page.getByLabel('Provider', { exact: true }).selectOption({ label: `${renamed} · custom` });
			await page.getByLabel('Model id').fill('e2e-endpoint');
			await page.getByRole('button', { name: 'Add model' }).click();
			await expect(page).toHaveURL(/\/admin\/models$/);
			const model = (await models(admin)).find((one) => one.label === `${renamed} model`)!;
			expect(model.connection_id).toBe(connection);

			await open(page, '/admin/models/providers');
			await expect(page.getByRole('region', { name: renamed }).getByText(`Used by ${renamed} model`)).toBeVisible();
			await page.getByRole('region', { name: renamed }).getByRole('button', { name: 'Disconnect' }).click();
			await page.getByRole('dialog').getByRole('button', { name: 'Disconnect' }).click();
			await expect(page.getByText(`Still used by ${renamed} model. Move or delete those models first.`)).toBeVisible();

			await deleteAdminModel(model.id);
			await open(page, '/admin/models/providers');
			await page.getByRole('region', { name: renamed }).getByRole('button', { name: 'Disconnect' }).click();
			await page.getByRole('dialog').getByRole('button', { name: 'Disconnect' }).click();
			await expect(page.getByText('Disconnected', { exact: true })).toBeVisible();
			await expect(page.getByRole('region', { name: renamed })).toHaveCount(0);
		} finally {
			const left = (await models(admin)).find((one) => one.label === `${renamed} model`);
			if (left) await deleteAdminModel(left.id);
			await deleteConnection(connection);
		}
	});

	test('the embedding model shows, or that none is saved here, and saves as it is', async ({ page, context }) => {
		const admin = await platformAdmin();
		const response = await api(admin, '/api/admin/settings/embedding');
		await signInAs(context, admin);
		await open(page, '/admin/models/embedding');

		if (response.status === 404) {
			// Nothing saved: the page says the environment's model is used, and saving would change that
			// for every test on this backend, so it isn't.
			await expect(page.getByText('No embedding model is saved here yet, so Nomi uses the one its environment sets. Save one to choose it here.')).toBeVisible();
			await expect(page.getByLabel('Model id')).toHaveValue('');
			return;
		}
		const before = (await response.json()) as { provider: string; model_id: string };
		await expect(page.getByLabel('Provider')).toHaveValue(before.provider);
		await expect(page.getByLabel('Model id')).toHaveValue(before.model_id);
		await page.getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Embedding model saved')).toBeVisible();
		expect(await (await api(admin, '/api/admin/settings/embedding')).json()).toMatchObject(before);
});
});

test.describe('tools', () => {
	type Tools = { groups: { tools: { name: string; enabled: boolean; settings: Record<string, unknown> | null }[] }[] };
	const tools = async (admin: Account) => ((await (await api(admin, '/api/admin/tools')).json()) as Tools).groups.flatMap((group) => group.tools);

	test('a tool is switched off and on again', async ({ page, context }) => {
		const admin = await platformAdmin();
		await signInAs(context, admin);
		await open(page, '/admin/tools');
		const switchable = page.getByRole('button', { name: /^Turn off \S+$/ }).first();
		const name = (await switchable.getAttribute('aria-label'))!.replace('Turn off ', '');
		try {
			await switchable.click();
			await expect(page.getByRole('button', { name: `Turn on ${name}` })).toBeVisible();
			expect((await tools(admin)).find((tool) => tool.name === name)?.enabled).toBe(false);
			await page.getByRole('button', { name: `Turn on ${name}` }).click();
			await expect(page.getByRole('button', { name: `Turn off ${name}` })).toBeVisible();
		} finally {
			await api(admin, `/api/admin/tools/${name}`, { method: 'PUT', body: JSON.stringify({ enabled: true }) });
		}
	});

	test('how much of a page is read is saved, and a number that isn’t one is refused', async ({ page, context }) => {
		const admin = await platformAdmin();
		const before = (await tools(admin)).find((tool) => tool.name === 'read_web_page')!.settings as { max_chars: number };
		await signInAs(context, admin);
		await open(page, '/admin/tools');
		const section = page.getByRole('region', { name: 'Reading a page' });
		try {
			await section.getByLabel('Characters').fill('lots');
			await section.getByRole('button', { name: 'Save' }).click();
			await expect(page.getByText('Write the numbers as numbers.')).toBeVisible();
			await section.getByLabel('Characters').fill('15 000');
			await section.getByRole('button', { name: 'Save' }).click();
			await expect(section.getByText('Saved', { exact: true })).toBeVisible();
			expect((await tools(admin)).find((tool) => tool.name === 'read_web_page')!.settings).toMatchObject({ max_chars: 15000 });
		} finally {
			await api(admin, '/api/admin/tools/read_web_page/settings', { method: 'PUT', body: JSON.stringify({ max_chars: before.max_chars }) });
		}
	});

	test('trying web search and hearing the voice say plainly what went wrong without a key', async ({ page, context }) => {
		const admin = await platformAdmin();
		await signInAs(context, admin);
		await open(page, '/admin/tools');

		await page.getByLabel('Question').fill('harga cabai hari ini');
		await page.getByRole('button', { name: 'Try it' }).click();
		await expect(page.getByText('The search didn’t work')).toBeVisible();

		await expect(page.getByText('The voices couldn’t be listed with the saved key, so write the voice id.')).toBeVisible();
		await page.getByRole('button', { name: 'Hear it' }).click();
		await expect(page.getByText('The voice couldn’t be heard')).toBeVisible();
	});
});

test.describe('custom agents', () => {
	test('an agent is made with its tools and look, switched off, and changed', async ({ page, context }) => {
		const admin = await platformAdmin();
		const name = `Juru Mudi ${Math.random().toString(36).slice(2, 7)}`;
		type Agent = { id: string; name: string; is_active: boolean; granted_tools: string[]; supports_todos: boolean; tone: string };
		const agents = async () => (await (await api(admin, '/api/admin/dynamic-agents')).json()) as Agent[];
		await signInAs(context, admin);
		await open(page, '/admin/dynamic-agents/new');
		try {
			await page.getByLabel('Name', { exact: true }).fill(name);
			// Something no other test asks about, so nothing is handed to it meanwhile.
			await page.getByLabel('What it’s for, in a few words').fill('Tide tables for fishing boats');
			await page.getByLabel('When Nomi should hand it a question').fill('Questions about tide tables for fishing boats at sea');
			await page.getByLabel('How it works').fill('Answer with the tide times for the harbour asked about.');
			await page.getByRole('checkbox', { name: 'list_transactions' }).click();
			await page.getByRole('checkbox', { name: 'Keep to-dos' }).click();
			await page.getByLabel('Colour').selectOption('tide');
			await page.getByRole('button', { name: 'Make agent' }).click();
			await expect(page).toHaveURL(/\/admin\/dynamic-agents$/);
			const made = (await agents()).find((one) => one.name === name)!;
			expect(made).toMatchObject({ granted_tools: ['list_transactions'], supports_todos: true, tone: 'tide' });

			const row = page.getByRole('row', { name: new RegExp(name) });
			if (made.is_active) {
				await row.getByRole('button', { name: `Turn off ${name}` }).click();
				await expect(row.getByRole('button', { name: `Turn on ${name}` })).toBeVisible();
			}
			expect((await agents()).find((one) => one.id === made.id)?.is_active).toBe(false);

			await row.getByRole('link', { name }).click();
			await page.getByRole('checkbox', { name: 'Make plans' }).click();
			await page.getByRole('button', { name: 'Save' }).click();
			await expect(page.getByText('Agent saved')).toBeVisible();
		} finally {
			const left = (await agents()).find((one) => one.name === name);
			if (left) await deleteDynamicAgent(left.id);
		}
	});
});

test.describe('support', () => {
	test('an article is written, found, unpublished and deleted', async ({ page, context }) => {
		const admin = await platformAdmin();
		const title = `Ongkir gratis ${Math.random().toString(36).slice(2, 7)}`;
		await signInAs(context, admin);
		await open(page, '/admin/support');

		await page.getByRole('link', { name: 'Write an article' }).click();
		await page.getByLabel('Title').fill(title);
		await page.getByLabel('Answer').fill('Gratis untuk belanja di atas Rp 100.000.');
		await page.getByLabel('Tags').fill('ongkir, pengiriman');
		await page.getByRole('button', { name: 'Save article' }).click();
		await expect(page).toHaveURL(/\/admin\/support$/);

		await page.getByRole('searchbox', { name: 'Search articles' }).fill(title);
		await page.getByRole('searchbox', { name: 'Search articles' }).press('Enter');
		await expect(page).toHaveURL(/\?q=/);
		await page.getByRole('link', { name: title }).click();
		await page.getByRole('switch', { name: 'Published' }).click();
		await page.getByRole('button', { name: 'Save' }).click();
		await expect(page.getByText('Article saved')).toBeVisible();
		type Listed = { title: string; published: boolean; tags: string[] }[];
		expect(((await (await api(admin, '/api/admin/support/articles')).json()) as Listed).find((one) => one.title === title)).toMatchObject({ published: false, tags: ['ongkir', 'pengiriman'] });

		await page.getByRole('button', { name: 'Delete article' }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Delete article' }).click();
		await expect(page).toHaveURL(/\/admin\/support$/);
		expect(((await (await api(admin, '/api/admin/support/articles')).json()) as Listed).some((one) => one.title === title)).toBe(false);
	});

	test('a key’s secret is shown once when it’s made, and the key is revoked', async ({ page, context }) => {
		const admin = await platformAdmin();
		const name = `Toko ${Math.random().toString(36).slice(2, 7)}`;
		await signInAs(context, admin);
		await open(page, '/admin/support');

		await page.getByLabel('Who uses it').fill(name);
		await page.getByLabel('Product (optional)').fill('Kasir');
		await page.getByRole('button', { name: 'Make a key' }).click();
		await expect(page.getByText('Key made. Copy the secret now: it isn’t shown again.')).toBeVisible();
		await expect(page.getByText(/^nsk_[0-9a-f]+$/)).toBeVisible();

		await open(page, '/admin/support');
		await expect(page.getByText(/^nsk_[0-9a-f]+$/)).toHaveCount(0);
		await page.getByRole('button', { name: `Revoke ${name}` }).click();
		await page.getByRole('dialog').getByRole('button', { name: 'Revoke' }).click();
		await expect(page.getByText('Key revoked')).toBeVisible();
		type Keys = { name: string; revoked_at: string | null }[];
		expect(((await (await api(admin, '/api/admin/support/keys')).json()) as Keys).find((one) => one.name === name)?.revoked_at).not.toBeNull();
	});
});
