import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { BACKEND_URL } from './support/env';

// The signed-in app shell against the real backend (App / Home Vr4nn, the account menu LkJLt and
// the workspace switcher t24cE): who's signed in, their counts, crew, theme, workspaces, and
// signing out, here or from another device.

const json = async <T>(response: Promise<Response>): Promise<T> => (await response).json() as Promise<T>;

/** The account's first workspace, made at sign-up. */
async function firstWorkspace(account: Account): Promise<{ id: string; name: string }> {
	return (await json<{ orgs: { id: string; name: string }[] }>(api(account, '/api/orgs'))).orgs[0];
}

const main = (page: Page) => page.getByRole('navigation', { name: 'Main' });

/** Opens the app once it has hydrated: before then a click reaches only the server's markup. */
async function open(page: Page, path = '/') {
	await page.goto(path, { waitUntil: 'networkidle' });
}

test('someone signed out signs in first, and comes back to where they were going', async ({ request }) => {
	for (const [path, back] of [
		['/', '%2F'],
		['/?from=email', '%2F%3Ffrom%3Demail'],
	]) {
		const response = await request.get(path, { maxRedirects: 0 });
		expect(response.status()).toBe(303);
		expect(response.headers().location).toBe(`/sign-in?redirect_to=${back}`);
	}
});

test('the sidebar shows the person, their workspace, their counts and recents, and the crew', async ({ page, context }) => {
	const account = await createAccount({ workspace: 'Kopi Kita' });
	await api(account, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Dinda Rahma' }) });
	const { session_id } = await json<{ session_id: string }>(api(account, '/api/sessions', { method: 'POST' }));
	await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Budget belanja Oktober' }) });
	const workspace = await firstWorkspace(account);
	expect((await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: workspace.id }) })).ok).toBe(true);

	await signInAs(context, account);
	await open(page);

	await expect(page.getByRole('heading', { level: 1 })).toHaveText(/^(Morning|Afternoon|Evening), Dinda\.$/);
	await expect(main(page).getByRole('button', { name: 'Dinda Rahma' })).toBeVisible();
	await expect(main(page).getByRole('button', { name: 'Projects from Kopi Kita' })).toBeVisible();
	await expect(main(page).getByRole('link', { name: /^Chats/ })).toHaveText(/Chats\s*1/);
	await expect(main(page).getByRole('link', { name: /^Projects/ })).toHaveText(/Projects\s*1/);
	await expect(main(page).getByRole('link', { name: /Home/ })).toHaveAttribute('aria-current', 'page');
	await expect(main(page).locator(`a[href="/chat/${session_id}"]`)).toHaveText(/now$/);
	await expect(main(page).getByRole('meter', { name: /plan$/ })).toBeVisible();
	await expect(page.getByText(/^(Mon|Tue|Wed|Thu|Fri|Sat|Sun), \d{1,2} \w{3,4} \d{4} · \d{2}:\d{2}$/)).toBeVisible();

	const { agents } = await json<{ agents: { name: string }[] }>(api(account, '/api/agents'));
	const crew = page.getByRole('complementary', { name: 'Your crew' });
	// The chat this test sent may still have Nomi at work: "10 idle", or "1 working · 9 idle".
	const head = crew.getByText(/^(\d+ working · )?\d+ idle$/);
	await expect(head).toBeVisible();
	const [, working = '0', idle] = (await head.textContent())!.match(/^(?:(\d+) working · )?(\d+) idle$/)!;
	expect(Number(working) + Number(idle)).toBe(agents.length);
	await expect(crew.getByRole('listitem')).toHaveCount(agents.length);
	for (const name of ['Nomi', 'Dana', 'Kala', 'Maya', 'Rena', 'Koda', 'Tara', 'Raga', 'Personality', 'Supervisor']) {
		await expect(crew.getByText(name, { exact: true })).toHaveCount(1);
	}
	// The ten the design has a mascot for wear it.
	await expect(crew.locator('li svg')).toHaveCount(10);
});

test('the theme picked in the account menu applies at once and stays after a reload', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page);

	await main(page).getByRole('button', { name: account.email.split('@')[0] }).click();
	await page.getByRole('menuitemradio', { name: 'Light' }).click();
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	await expect.poll(async () => (await json<{ theme: string }>(api(account, '/api/preferences'))).theme).toBe('light');

	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	await main(page).getByRole('button', { name: account.email.split('@')[0] }).click();
	await expect(page.getByRole('menuitemradio', { name: 'Light' })).toHaveAttribute('aria-checked', 'true');

	await page.getByRole('menuitemradio', { name: 'System' }).click();
	await expect(page.locator('html')).not.toHaveAttribute('data-theme', /./);
	await expect.poll(async () => (await json<{ theme: string }>(api(account, '/api/preferences'))).theme).toBe('system');
});

test('a new workspace can be made, then picked for projects, and that stays', async ({ page, context }) => {
	const account = await createAccount({ workspace: 'E2E space' });
	await signInAs(context, account);
	await open(page);

	await main(page).getByRole('button', { name: 'Projects from E2E space' }).click();
	await page.getByRole('menuitem', { name: 'Create workspace' }).click();
	const dialog = page.getByRole('dialog', { name: 'Create a workspace' });
	await dialog.getByRole('button', { name: 'Create' }).click();
	await expect(dialog.getByText('Give the workspace a name.')).toBeVisible();
	await dialog.getByRole('textbox', { name: 'Name' }).fill('Warung Dinda');
	await dialog.getByRole('textbox', { name: 'Name' }).press('Enter');
	await expect(dialog).toBeHidden();
	await expect(page.getByRole('status').filter({ hasText: 'Created Warung Dinda' })).toBeVisible();

	// Made, but projects still come from the first one until it's picked.
	await main(page).getByRole('button', { name: 'Projects from E2E space' }).click();
	await expect(page.getByRole('menuitemradio', { name: /E2E space/ })).toHaveAttribute('aria-checked', 'true');
	await page.getByRole('menuitemradio', { name: /Warung Dinda/ }).click();
	await expect(main(page).getByRole('button', { name: 'Projects from Warung Dinda' })).toBeVisible();
	await expect(main(page).getByRole('link', { name: /^Projects/ })).toHaveText(/Projects\s*0/);

	const created = (await json<{ orgs: { id: string; name: string }[]; project_org_id: string }>(api(account, '/api/orgs'))).orgs.find((org) => org.name === 'Warung Dinda');
	expect((await json<{ project_org_id: string }>(api(account, '/api/orgs'))).project_org_id).toBe(created?.id);

	await page.reload({ waitUntil: 'networkidle' });
	await expect(main(page).getByRole('button', { name: 'Projects from Warung Dinda' })).toBeVisible();
	await main(page).getByRole('button', { name: 'Projects from Warung Dinda' }).click();
	await expect(page.getByRole('menuitemradio', { name: /Warung Dinda/ })).toHaveAttribute('aria-checked', 'true');
	await expect(page.getByRole('menuitem', { name: /Workspace settings & members/ })).toHaveAttribute('href', `/settings/workspaces/${created?.id}`);
});

test('logging out ends the sign-in: it lands on sign in, and the app asks for it again', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page);

	await main(page).getByRole('button', { name: account.email.split('@')[0] }).click();
	await page.getByRole('menuitem', { name: 'Log out' }).click();
	await expect(page).toHaveURL(/\/sign-in$/);
	expect((await context.cookies()).map((cookie) => cookie.name)).not.toContain('refresh_token');

	// The backend no longer renews it.
	const refreshed = await fetch(`${BACKEND_URL}/api/auth/refresh`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ refresh_token: account.tokens.refresh_token }) });
	expect(refreshed.status).toBe(401);

	const response = await page.request.get('/', { maxRedirects: 0 });
	expect(response.headers().location).toBe('/sign-in?redirect_to=%2F');
});

test('signed out from another device, this one signs out too', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await open(page);
	// This browser is in the person's device list once the app has loaded.
	const deviceId = (await context.cookies()).find((cookie) => cookie.name === 'device_id')?.value;
	await expect.poll(async () => (await json<{ id: string }[]>(api(account, '/api/devices'))).map((device) => device.id)).toContain(deviceId);
	// The live feed is open (it greets nothing, so wait for the shell to settle).
	await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
	await page.waitForTimeout(1000);

	expect((await api(account, `/api/devices/${deviceId}`, { method: 'DELETE' })).status).toBe(204);
	await expect(page).toHaveURL(/\/sign-in$/, { timeout: 15_000 });
	expect((await context.cookies()).map((cookie) => cookie.name)).not.toContain('access_token');
});
