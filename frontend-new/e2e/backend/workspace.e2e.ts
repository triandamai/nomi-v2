import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// A project's workspace against the real backend (Flow / Build — Sign in → live URL o882RC,
// Building and Live): started from Projects it opens in its workspace with Koda on it; the chat
// pane carries on the conversation; Code lists the files; a static site shows in the preview, on a
// desktop or a phone; the backend's old links lead here.

const REPLY = 'This is a fake response for local development and testing.';
type Projects = { projects: { id: string; session_id: string }[] };

test('a project opens in its workspace: chat, files and the preview', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/projects', { waitUntil: 'networkidle' });
	await page.getByRole('textbox', { name: 'Describe an app to build' }).fill('A one-page menu for Warung Kopi');
	await page.getByRole('button', { name: 'Start building' }).click();
	await expect(page).toHaveURL(/\/projects\/[0-9a-f-]{36}$/);
	const { id, session_id } = ((await (await api(account, '/api/projects')).json()) as Projects).projects[0];

	// The bar: back, the project, where it lives; no sidebar.
	await expect(page.getByRole('banner').getByText('New project', { exact: true })).toBeVisible();
	await expect(page.getByRole('link', { name: 'Back to projects' })).toHaveAttribute('href', '/projects');
	await expect(page.getByRole('navigation', { name: 'Main' })).toHaveCount(0);

	// The chat pane: the description, Koda's answer, and the conversation carrying on.
	const chat = page.getByRole('region', { name: 'Chat' });
	await expect(chat.getByText('A one-page menu for Warung Kopi')).toBeVisible();
	await expect(chat.getByText(REPLY)).toHaveCount(1);
	const composer = chat.getByRole('textbox', { name: 'Message' });
	await expect(composer).toHaveAttribute('placeholder', 'Ask about this project…');
	await expect(async () => {
		await composer.fill('Tambah jam buka');
		await expect(chat.getByRole('button', { name: 'Send' })).toBeEnabled({ timeout: 1_000 });
	}).toPass();
	await chat.getByRole('button', { name: 'Send' }).click();
	await expect(chat.getByText(REPLY)).toHaveCount(2);

	// Nothing to show yet; no files yet.
	await expect(page.getByText('Nothing to preview yet')).toBeVisible();
	await page.getByRole('radio', { name: 'Code' }).click();
	await expect(page.getByText('No files yet')).toBeVisible();
	expect(await page.getByRole('radio', { name: 'Terminal' }).count()).toBe(0);

	// A static site, imported: listed under Code, and shown in the preview.
	const imported = await api(account, `/api/projects/${id}/import/files`, { method: 'POST', body: JSON.stringify({ files: [{ path: 'index.html', content: '<h1>Warung Kopi Bu Siti</h1>' }], finish: true }) });
	expect(imported.ok).toBe(true);
	await page.reload({ waitUntil: 'networkidle' });
	await page.getByRole('radio', { name: 'Code' }).click();
	await expect(page.getByRole('treeitem', { name: /index\.html/ })).toBeVisible();
	await expect(page.getByText('<h1>Warung Kopi Bu Siti</h1>')).toBeVisible();
	await expect(page.getByText('read-only · 1 line')).toBeVisible();

	// Lines asked about: quoted above the composer, then sent ahead of the message.
	await page.getByRole('button', { name: 'Line 1' }).click();
	await page.getByRole('button', { name: 'Ask Koda about lines 1' }).click();
	await expect(chat.getByText('L1')).toBeVisible();
	await expect(async () => {
		await composer.fill('Make this bigger');
		await expect(chat.getByRole('button', { name: 'Send' })).toBeEnabled({ timeout: 1_000 });
	}).toPass();
	await chat.getByRole('button', { name: 'Send' }).click();
	await expect(chat.getByText('Make this bigger')).toBeVisible();
	await expect(chat.getByText(/About .?index\.html.? line 1:/)).toBeVisible();
	await expect(chat.getByText('L1')).toHaveCount(0);
	await page.getByRole('radio', { name: 'Preview' }).click();
	const preview = page.frameLocator(`iframe[src="/projects/${id}/preview/"]`);
	await expect(preview.getByRole('heading', { name: 'Warung Kopi Bu Siti' })).toBeVisible();
	await page.getByRole('radio', { name: 'Phone' }).click();
	await expect(page.frameLocator(`iframe[src="/projects/${id}/preview/"]`).getByRole('heading', { name: 'Warung Kopi Bu Siti' })).toBeVisible();

	// The backend's links, and the Projects card, lead here.
	await page.goto(`/projects/session/${session_id}`);
	await expect(page).toHaveURL(new RegExp(`/projects/${id}$`));
	await page.goto(`/projects/${id}/app`);
	await expect(page).toHaveURL(new RegExp(`/projects/${id}$`));
	await page.getByRole('link', { name: 'Back to projects' }).click();
	await expect(page).toHaveURL(/\/projects$/);
});

test('a project that isn’t theirs says so', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/projects/00000000-0000-0000-0000-000000000000');
	await expect(page.getByText('This project isn’t there, or you’re not in its workspace.')).toBeVisible();
	await page.getByRole('main').getByRole('link', { name: 'Projects' }).click();
	await expect(page).toHaveURL(/\/projects$/);
});

test('Deployments, Domain, Monitor and Database say what can’t be done yet, and nothing offers to', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const org = ((await (await api(account, '/api/orgs')).json()) as { orgs: { id: string }[] }).orgs[0].id;
	const made = (await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as { project_id: string };
	await page.goto(`/projects/${made.project_id}`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('button', { name: /^Publish/ })).toHaveCount(0);
	const tools = page.getByRole('navigation', { name: 'Project tools' });
	await tools.getByRole('button', { name: 'Deployments' }).click();
	await expect(tools.getByRole('button', { name: 'Deployments' })).toHaveAttribute('aria-pressed', 'true');
	await expect(page.getByText('Publishing isn’t set up here')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(page.getByText('Publishing isn’t set up here')).toHaveCount(0);

	// Domain: nothing published, so no address yet, and nothing to change.
	await tools.getByRole('button', { name: 'Domain' }).click();
	await expect(page.getByText('No address yet')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Change address' })).toHaveCount(0);
	await tools.getByRole('button', { name: 'Domain' }).click();
	await expect(page.getByText('No address yet')).toHaveCount(0);

	// Monitor: nothing running to measure yet.
	await tools.getByRole('button', { name: 'Monitor' }).click();
	await expect(page.getByText('Nothing running yet')).toBeVisible();
	await page.getByRole('button', { name: 'Close' }).click();
	await expect(page.getByText('Nothing running yet')).toHaveCount(0);

	// Database: databases live where apps are published, which isn't set up here.
	await tools.getByRole('button', { name: 'Database' }).click();
	await expect(page.getByRole('region', { name: 'Database' }).getByText('Publishing isn’t set up here')).toBeVisible();
});


test('Settings: transfer to another workspace, and delete after typing the name', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const orgs = async () => ((await (await api(account, '/api/orgs')).json()) as { orgs: { id: string; name: string }[] }).orgs;
	const home = (await orgs())[0].id;
	expect((await api(account, '/api/orgs', { method: 'POST', body: JSON.stringify({ name: 'Kopi Kita' }) })).ok).toBe(true);
	const kopi = (await orgs()).find((org) => org.name === 'Kopi Kita')!.id;
	const make = async () => ((await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: home }) })).json()) as { project_id: string }).project_id;

	// Transfer: pick the workspace, type the name, and it's gone from here.
	const moving = await make();
	await page.goto(`/projects/${moving}`, { waitUntil: 'networkidle' });
	const tools = page.getByRole('navigation', { name: 'Project tools' });
	await tools.getByRole('button', { name: 'Settings' }).click();
	await expect(page.getByText('Nothing is live to take down.')).toBeVisible();
	await page.getByRole('button', { name: 'Transfer…' }).click();
	const transfer = page.getByRole('dialog', { name: 'Transfer New project?' });
	await transfer.getByText('Kopi Kita').click();
	await expect(transfer.getByRole('button', { name: 'Transfer' })).toBeDisabled();
	await transfer.getByRole('textbox', { name: 'Type New project to confirm' }).fill('New project');
	await transfer.getByRole('button', { name: 'Transfer' }).click();
	await expect(page).toHaveURL(/\/projects$/);
	await expect(page.getByText('New project moved to its new workspace')).toBeVisible();
	const there = (await (await api(account, `/api/projects?org_id=${kopi}`)).json()) as { projects: { id: string }[] };
	expect(there.projects.map((one) => one.id)).toContain(moving);

	// Delete: only once the name is typed exactly.
	const going = await make();
	await page.goto(`/projects/${going}`, { waitUntil: 'networkidle' });
	await tools.getByRole('button', { name: 'Settings' }).click();
	await page.getByRole('button', { name: 'Delete…' }).click();
	const remove = page.getByRole('dialog', { name: 'Delete New project?' });
	await expect(remove.getByRole('link', { name: 'Export repo' })).toHaveAttribute('href', `/projects/${going}/export`);
	await remove.getByRole('textbox', { name: 'Type New project to confirm' }).fill('New projec');
	await expect(remove.getByRole('button', { name: 'Delete forever' })).toBeDisabled();
	await remove.getByRole('textbox', { name: 'Type New project to confirm' }).fill('New project');
	await remove.getByRole('button', { name: 'Delete forever' }).click();
	await expect(page).toHaveURL(/\/projects$/);
	await expect(page.getByText('New project deleted')).toBeVisible();
	expect((await api(account, `/api/projects/${going}`)).status).toBe(404);
});

test('People and Environment: roles, access, an invite, and variables', async ({ page, context }) => {
	const owner = await createAccount();
	const sari = await createAccount();
	const home = ((await (await api(owner, '/api/orgs')).json()) as { orgs: { id: string }[] }).orgs[0].id;
	const code = ((await (await api(owner, `/api/orgs/${home}/invites`, { method: 'POST', body: JSON.stringify({ email: sari.email, role: 'member' }) })).json()) as { code: string }).code;
	expect((await api(sari, `/api/invites/${code}/accept`, { method: 'POST' })).ok).toBe(true);
	const id = ((await (await api(owner, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: home }) })).json()) as { project_id: string }).project_id;
	type Members = { access: string; people: { user_id: string; name: string; project_role: string | null; is_owner: boolean }[] };
	const members = async () => (await (await api(owner, `/api/projects/${id}/members`)).json()) as Members;
	const sariName = (await members()).people.find((one) => !one.is_owner)!.name;

	await signInAs(context, owner);
	await page.goto(`/projects/${id}`, { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Project tools' }).getByRole('button', { name: 'Settings' }).click();
	await page.getByRole('button', { name: 'Manage' }).first().click();
	const sheet = page.getByRole('region', { name: 'People' });

	// Sari opens it through the workspace; made a viewer, then taken off again.
	await sheet.getByRole('combobox', { name: `${sariName}’s role` }).selectOption('viewer');
	await expect(page.getByText(`${sariName}’s role changed`)).toBeVisible();
	await expect.poll(async () => (await members()).people.find((one) => one.name === sariName)?.project_role).toBe('viewer');
	await sheet.getByRole('button', { name: `Remove ${sariName}` }).click();
	await expect(page.getByText(`${sariName} removed from the project`)).toBeVisible();
	await expect.poll(async () => (await members()).people.find((one) => one.name === sariName)?.project_role ?? null).toBeNull();

	// Only the people added from now on.
	await sheet.getByRole('radio', { name: 'Only people added' }).click();
	await expect(page.getByText('Only people added can open it now')).toBeVisible();
	await expect.poll(async () => (await members()).access).toBe('members');

	// An invite by address.
	await sheet.getByRole('textbox', { name: 'Email address' }).fill('budi@kopi.id');
	await sheet.getByRole('button', { name: 'Invite' }).click();
	await expect(page.getByText('budi@kopi.id invited')).toBeVisible();

	// Environment: a proper name, then the backend's reason (no Shipyard here to keep it).
	await sheet.getByRole('button', { name: 'Close' }).click();
	await page.getByRole('button', { name: 'Manage' }).nth(1).click();
	const env = page.getByRole('region', { name: 'Environment' });
	await expect(env.getByText('No variables yet.', { exact: false })).toBeVisible();
	await env.getByRole('textbox', { name: 'Name' }).fill('1bad');
	await env.getByLabel('Value').fill('1234');
	await env.getByRole('button', { name: 'Add variable' }).click();
	await expect(env.getByText('Capitals, digits and underscores, starting with a letter (ADMIN_PIN).')).toBeVisible();
	await env.getByRole('textbox', { name: 'Name' }).fill('ADMIN_PIN');
	await env.getByRole('button', { name: 'Add variable' }).click();
	await expect(env.getByText(/isn't set up|not set up/i)).toBeVisible();
});

test('Export says what it holds and pushes to GitHub; Share waits for a live app', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const home = ((await (await api(account, '/api/orgs')).json()) as { orgs: { id: string }[] }).orgs[0].id;
	const id = ((await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: home }) })).json()) as { project_id: string }).project_id;
	await api(account, `/api/projects/${id}/import/files`, { method: 'POST', body: JSON.stringify({ files: [{ path: 'index.html', content: '<h1>Warung</h1>' }], finish: true }) });
	await page.goto(`/projects/${id}`, { waitUntil: 'networkidle' });

	await expect(page.getByRole('banner').getByRole('button', { name: 'Share' })).toHaveCount(0);

	await page.getByRole('button', { name: 'Export repo' }).click();
	const exporting = page.getByRole('dialog', { name: /Export/ });
	await expect(exporting.getByText(/^1 file ·/)).toBeVisible();
	await exporting.getByRole('button', { name: /GitHub/ }).click();
	const push = page.getByRole('dialog', { name: 'Push to GitHub' });
	await push.getByRole('textbox', { name: 'Repository' }).fill('not a repo');
	await push.getByRole('button', { name: 'Push' }).click();
	await expect(push.getByText('Name the repository as owner/name.')).toBeVisible();
	// No GitHub connected here: the backend says so, under the field.
	await push.getByRole('textbox', { name: 'Repository' }).fill('mario/warung-kopi');
	await push.getByRole('button', { name: 'Push' }).click();
	await expect(push.getByText(/GitHub/i).last()).toBeVisible();
	await expect(push.getByRole('textbox', { name: 'Repository' })).toHaveAttribute('aria-invalid', 'true');
});
