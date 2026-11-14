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
	await expect(page.getByText('index.html')).toBeVisible();
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

