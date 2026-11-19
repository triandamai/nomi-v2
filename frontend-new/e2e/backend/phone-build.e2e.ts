import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { seedPublish } from './support/seed';

// A project's workspace on a phone against the real backend (Flow / Mobile Build; contract
// docs/ux/work/phone-build/40-ux-contract.md): the phone's own screen below 768 (nav, Chat |
// Preview, the Ask bar), every tool from ··· as a sheet, Code and Terminal pushed, invites with
// their link, a failed publish's card and log, two people in one project, and the deep links.
// No Shipyard runs here: publishing can't start, so the publish pill is the component tests'.

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

type Orgs = { orgs: { id: string }[] };

/** A project with a static site in it, made as Projects makes one. */
async function siteProject(account: Account, files = [{ path: 'index.html', content: '<h1>Warung Kopi Bu Siti</h1>\n<p>Kopi susu gula aren</p>' }]): Promise<string> {
	const org = ((await (await api(account, '/api/orgs')).json()) as Orgs).orgs[0].id;
	const id = ((await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as { project_id: string }).project_id;
	expect((await api(account, `/api/projects/${id}`, { method: 'PATCH', body: JSON.stringify({ name: 'warung-kopi-menu' }) })).ok).toBe(true);
	expect((await api(account, `/api/projects/${id}/import/files`, { method: 'POST', body: JSON.stringify({ files, finish: true }) })).ok).toBe(true);
	return id;
}

async function open(page: Page, context: BrowserContext, account: Account, path: string) {
	await signInAs(context, account);
	await page.goto(path, { waitUntil: 'networkidle' });
}

test('below 768 the workspace is the phone’s screen: the nav, Chat | Preview, the Ask bar, and no desktop split', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await open(page, context, mario, `/projects/${id}`);

	// The phone's nav: back, the project in mono, ···; no desktop bar or tabs.
	await expect(page.getByRole('heading', { level: 1, name: 'warung-kopi-menu' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Project options' })).toBeVisible();
	await expect(page.getByRole('link', { name: 'Back to projects' })).toHaveCount(0);
	await expect(page.getByRole('navigation', { name: 'Project tools' })).toHaveCount(0);
	const switcher = page.getByRole('radiogroup', { name: 'Chat or preview' });
	await expect(switcher.getByRole('radio', { name: 'Chat' })).toHaveAttribute('aria-checked', 'true');
	// Only one layout is in the page: one composer.
	await expect(page.getByRole('textbox', { name: 'Message' })).toHaveCount(1);

	// Preview: the site full width, the Ask bar floating; Phone / Desktop.
	await switcher.getByRole('radio', { name: 'Preview' }).click();
	const frame = page.frameLocator(`iframe[src^="/projects/${id}/preview/~"]`);
	await expect(frame.getByRole('heading', { name: 'Warung Kopi Bu Siti' })).toBeVisible();
	const box = await page.locator(`iframe[src^="/projects/${id}/preview/~"]`).boundingBox();
	expect(box?.width).toBe(390);
	// The preview's own bar (PsC5F, Preview Bar GppXE): the address, Reload, and Phone | Desktop; no
	// second switch under Chat | Preview.
	await expect(page.getByRole('button', { name: 'Reload' })).toBeVisible();
	await expect(page.getByRole('radio', { name: 'Phone' })).toHaveAttribute('aria-checked', 'true');
	await page.getByRole('radio', { name: 'Desktop' }).click();
	await expect(page.getByText('1440 px wide, scaled to fit · pinch to zoom')).toBeVisible();
	// The desktop layout sits in the pen's 240-high card, not stretched down the screen.
	const card = await page.locator(`iframe[src^="/projects/${id}/preview/~"]`).locator('xpath=../..').boundingBox();
	expect(card?.height).toBe(240);
	await expect(page.locator(`iframe[src^="/projects/${id}/preview/~"]`)).toHaveAttribute('style', /width: 1440px/);

	// The Ask bar goes to the chat with the composer ready.
	await page.getByRole('button', { name: 'Ask Koda for a change…' }).click();
	await expect(switcher.getByRole('radio', { name: 'Chat' })).toHaveAttribute('aria-checked', 'true');
	await expect(page.getByRole('textbox', { name: 'Message' })).toBeFocused();

	// Nothing on the page is wider than the phone.
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);

	// Back goes to Projects.
	await page.getByRole('button', { name: 'Back', exact: true }).click();
	await expect(page).toHaveURL(/\/projects$/);
});

test('··· lists the tools the desktop shows; Deployments says publishing isn’t set up; Export says what it holds', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await open(page, context, mario, `/projects/${id}`);

	await page.getByRole('button', { name: 'Project options' }).click();
	const tools = page.getByRole('dialog', { name: 'warung-kopi-menu' });
	await expect(tools.getByRole('button', { name: /^Deployments/ })).toBeVisible();
	await expect(tools.getByRole('button', { name: /^Settings/ })).toBeVisible();
	await expect(tools.getByRole('button', { name: /^Code/ })).toBeVisible();
	// A site without tables has no Database, and says why; no sandbox, no Terminal; not live, no Share.
	await expect(tools.getByRole('button', { name: /^Database/ })).toHaveCount(0);
	await expect(tools.getByText('No Database: this site doesn’t store data.')).toBeVisible();
	await expect(tools.getByRole('button', { name: /^Terminal/ })).toHaveCount(0);
	await expect(tools.getByRole('button', { name: 'Share' })).toHaveCount(0);

	await tools.getByRole('button', { name: /^Deployments/ }).click();
	const deployments = page.getByRole('dialog', { name: 'Deployments' });
	await expect(deployments.getByText('Publishing isn’t set up here')).toBeVisible();
	await deployments.getByRole('button', { name: 'Close' }).click();
	await expect(deployments).toHaveCount(0);

	await page.getByRole('button', { name: 'Project options' }).click();
	await page.getByRole('dialog', { name: 'warung-kopi-menu' }).getByRole('button', { name: 'Export' }).click();
	const exporting = page.getByRole('dialog', { name: 'Export warung-kopi-menu' });
	await expect(exporting.getByText('included on Free · no fee')).toBeVisible();
	await expect(exporting.getByText(/1 file/)).toBeVisible();
	await expect(exporting.getByRole('button', { name: 'Push to GitHub' })).toBeVisible();
	await expect(exporting.getByRole('button', { name: 'Download zip' })).toBeVisible();
	await exporting.getByRole('button', { name: 'Push to GitHub' }).click();
	await expect(page.getByRole('dialog', { name: 'Push to GitHub' })).toBeVisible();
});

test('Code: the tree, a file, lines picked and asked about from the chat', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario, [
		{ path: 'index.html', content: '<h1>Warung Kopi Bu Siti</h1>\n<p>Kopi susu gula aren</p>\n<p>Es kopi</p>' },
		{ path: 'menu/items.json', content: '[]' },
	]);
	await open(page, context, mario, `/projects/${id}`);
	await page.getByRole('button', { name: 'Project options' }).click();
	await page.getByRole('dialog', { name: 'warung-kopi-menu' }).getByRole('button', { name: /^Code/ }).click();

	// The pushed screen: back, the tree as rows, the note.
	await expect(page.getByRole('heading', { level: 1, name: 'Code' })).toBeVisible();
	await expect(page.getByText('Read-only. Ask Koda to change anything.')).toBeVisible();
	await page.getByRole('button', { name: /^menu/ }).click();
	await expect(page.getByRole('button', { name: /menu\/items\.json/ })).toBeVisible();
	await page.getByRole('button', { name: /^index\.html/ }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'index.html' })).toBeVisible();

	// Lines picked (Enter, then Shift+↓), then asked about: quoted above the composer on Chat.
	await page.getByRole('button', { name: 'Line 2' }).focus();
	await page.keyboard.press('Enter');
	await page.keyboard.press('Shift+ArrowDown');
	await expect(page.getByRole('button', { name: 'Line 3' })).toHaveAttribute('aria-pressed', 'true');
	await page.getByRole('button', { name: 'Ask Koda about lines 2–3' }).click();
	await expect(page.getByRole('radio', { name: 'Chat' })).toHaveAttribute('aria-checked', 'true');
	await expect(page.getByText('L2–3')).toBeVisible();
	const composer = page.getByRole('textbox', { name: 'Message' });
	await expect(async () => {
		await composer.fill('Make these bigger');
		await expect(page.getByRole('button', { name: 'Send' })).toBeEnabled({ timeout: 1_000 });
	}).toPass();
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText(/About .?index\.html.? lines 2–3:/)).toBeVisible();
	await expect(page.getByText('L2–3')).toHaveCount(0);
});

test('Settings: People invites with a link to copy, Environment says the backend’s reason, Transfer and Delete', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await context.grantPermissions(['clipboard-read', 'clipboard-write']);
	await open(page, context, mario, `/projects/${id}`);
	await page.getByRole('button', { name: 'Project options' }).click();
	await page.getByRole('dialog', { name: 'warung-kopi-menu' }).getByRole('button', { name: /^Settings/ }).click();
	const settings = page.getByRole('dialog', { name: 'Settings' });
	await expect(settings.getByRole('button', { name: /^Transfer ownership/ })).toBeVisible();
	await expect(settings.getByRole('button', { name: /^Take down/ })).toHaveCount(0);
	// No Name row (D3).
	await expect(settings.getByRole('button', { name: /^Name/ })).toHaveCount(0);

	// People: the address, then what they may do, then the link to send in case the email is lost.
	await settings.getByRole('button', { name: /^People/ }).click();
	const people = page.getByRole('dialog', { name: 'People' });
	await people.getByRole('textbox', { name: 'Email address' }).fill('sari.w@kopi.id');
	await people.getByRole('button', { name: 'Invite', exact: true }).click();
	const inviting = page.getByRole('dialog', { name: 'Invite to warung-kopi-menu' });
	await inviting.getByRole('button', { name: /^Can prompt/ }).click();
	await expect(inviting.getByRole('button', { name: /^Can prompt/ })).toHaveAttribute('aria-pressed', 'true');
	await inviting.getByRole('button', { name: 'Send invite' }).click();
	const back = page.getByRole('dialog', { name: 'People' });
	await expect(back.getByText('Invite sent')).toBeVisible();
	await expect(back.getByText('to sari.w@kopi.id · 7 days')).toBeVisible();
	await back.getByRole('button', { name: 'Copy link' }).click();
	expect(await page.evaluate(() => navigator.clipboard.readText())).toMatch(/\/invite\/[\w-]+$/);

	// Closing People goes back to Settings; Environment says why a variable can't be kept here.
	await back.getByRole('button', { name: 'Close' }).click();
	await page.getByRole('dialog', { name: 'Settings' }).getByRole('button', { name: /^Environment/ }).click();
	const env = page.getByRole('dialog', { name: 'Environment' });
	await env.getByRole('button', { name: 'Add variable' }).click();
	await env.getByRole('textbox', { name: 'Name' }).fill('ADMIN_PIN');
	await env.getByLabel('Value').fill('1234');
	await env.getByRole('button', { name: 'Add variable' }).click();
	await expect(env.getByText(/isn't set up|not set up/i)).toBeVisible();
	await env.getByRole('button', { name: 'Close' }).click();

	// Transfer: nowhere else to go; Delete wants the name, and Projects says it's gone.
	await page.getByRole('dialog', { name: 'Settings' }).getByRole('button', { name: /^Transfer ownership/ }).click();
	await expect(page.getByRole('dialog', { name: 'Transfer warung-kopi-menu?' }).getByText('No other workspace to move it to. Create or join one first.')).toBeVisible();
	await page.getByRole('dialog', { name: 'Transfer warung-kopi-menu?' }).getByRole('button', { name: 'Close' }).click();
	await page.getByRole('dialog', { name: 'Settings' }).getByRole('button', { name: /^Delete project/ }).click();
	const deleting = page.getByRole('dialog', { name: 'Delete warung-kopi-menu?' });
	await expect(deleting.getByRole('button', { name: 'Delete project' })).toBeDisabled();
	await deleting.getByRole('textbox').fill('warung-kopi-menu');
	await deleting.getByRole('button', { name: 'Delete project' }).click();
	await expect(page).toHaveURL(/\/projects\?deleted=|\/projects$/);
});

test('a failed publish: the card with the backend’s reason, Fix it, and the log; nothing live yet says so', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await seedPublish(id, { version: 1, status: 'failed', error: 'build failed', failure: { step: 'build', phase: 'build', message: "Cannot find module './PickupSlots.svelte'", tail: ['$ npm run build', 'vite v5.4.0 building for production...', "error: Cannot find module './PickupSlots.svelte'"] } });
	await open(page, context, mario, `/projects/${id}`);

	await expect(page.getByText('Publishing stopped at the build step. Nothing is live yet.')).toBeVisible();
	const card = page.getByRole('alert').filter({ hasText: 'Build failed' });
	await expect(card.getByText("Cannot find module './PickupSlots.svelte'")).toBeVisible();
	await expect(card.getByText(/^deploy #1 · build step · 00:41$/)).toBeVisible();

	await card.getByRole('button', { name: 'View log' }).click();
	await expect(page.getByRole('heading', { level: 1, name: 'Publish log' })).toBeVisible();
	await expect(page.getByRole('log').getByText('vite v5.4.0 building for production...')).toBeVisible();
	await page.getByRole('button', { name: 'Back', exact: true }).click();

	await page.getByRole('alert').filter({ hasText: 'Build failed' }).getByRole('button', { name: 'Fix it' }).click();
	await expect(page.getByRole('textbox', { name: 'Message' })).toBeFocused();
	await expect(page.getByText('deploy #1').first()).toBeVisible();
});

test('a live app: the nav says which deploy, Deployments lists them, the live one first', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await seedPublish(id, { version: 1, status: 'succeeded', url: 'https://warung-kopi-menu.example.test', minutesAgo: 30 });
	await seedPublish(id, { version: 2, status: 'failed', error: 'exit 1', failure: { step: 'build', message: 'exit 1', tail: [] }, minutesAgo: 5 });
	await open(page, context, mario, `/projects/${id}`);

	await expect(page.getByText('live · deploy #1')).toBeVisible();
	await expect(page.getByText('Publishing stopped at the build step. Your live site is still deploy #1; nothing changed for customers.')).toBeVisible();
	await page.getByRole('button', { name: 'Project options' }).click();
	const tools = page.getByRole('dialog', { name: 'warung-kopi-menu' });
	await expect(tools.getByRole('button', { name: 'Share' })).toBeVisible();
	await tools.getByRole('button', { name: 'Share' }).click();
	const share = page.getByRole('dialog', { name: 'Share warung-kopi-menu' });
	await expect(share.getByText('anyone with the link can open it')).toBeVisible();
	await expect(share.getByRole('link', { name: 'WhatsApp' })).toHaveAttribute('href', /^https:\/\/wa\.me\/\?text=https%3A%2F%2Fwarung-kopi-menu\.example\.test/);
	await expect(share.getByRole('link', { name: 'Telegram' })).toHaveAttribute('href', /^https:\/\/t\.me\/share\/url\?url=/);
	await expect(share.getByRole('link', { name: 'Mail' })).toHaveAttribute('href', /^mailto:\?body=/);
});

test('?open= opens the same sheet or screen on a phone, then tidies the address', async ({ page, context }) => {
	const mario = await createAccount();
	const id = await siteProject(mario);
	await open(page, context, mario, `/projects/${id}?open=settings`);
	await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible();
	await expect(page).toHaveURL(new RegExp(`/projects/${id}$`));
	await page.goto(`/projects/${id}?open=code&file=index.html`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('heading', { level: 1, name: 'index.html' })).toBeVisible();
	await page.goto(`/projects/${id}?open=export`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('dialog', { name: 'Export warung-kopi-menu' })).toBeVisible();
	// The backend's notification link lands on the workspace's Chat tab.
	await page.goto(`/projects/${id}/app`, { waitUntil: 'networkidle' });
	await expect(page.getByRole('radio', { name: 'Chat' })).toHaveAttribute('aria-checked', 'true');
});

test('two people on phones: the strip says who’s here and where, and Follow goes where she goes', async ({ page, context, browser }) => {
	const mario = await createAccount();
	const sari = await createAccount();
	await api(mario, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Mario Prasetyo' }) });
	await api(sari, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Sari Wulandari' }) });
	const org = ((await (await api(mario, '/api/orgs')).json()) as Orgs).orgs[0].id;
	const invite = (await (await api(mario, `/api/orgs/${org}/invites`, { method: 'POST', body: JSON.stringify({ email: sari.email, role: 'member' }) })).json()) as { code: string };
	expect((await api(sari, `/api/invites/${invite.code}/accept`, { method: 'POST' })).ok).toBe(true);
	const id = await siteProject(mario);

	await open(page, context, mario, `/projects/${id}`);
	const theirs = await browser.newContext({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });
	const sariPage = await theirs.newPage();
	await open(sariPage, theirs, sari, `/projects/${id}`);

	await expect(page.getByText('Sari Wulandari is here · Preview')).toBeVisible();
	await page.getByRole('button', { name: /Sari Wulandari is here/ }).click();
	const here = page.getByRole('dialog', { name: 'In this project' });
	await expect(here.getByText('2 people · live now')).toBeVisible();
	await expect(here.getByText('Viewing Preview')).toBeVisible();

	// Sari opens Code on a file; Mario follows her there.
	await sariPage.getByRole('button', { name: 'Project options' }).click();
	await sariPage.getByRole('dialog', { name: 'warung-kopi-menu' }).getByRole('button', { name: /^Code/ }).click();
	await sariPage.getByRole('button', { name: /^index\.html/ }).click();
	await expect(here.getByText('Viewing Code · index.html')).toBeVisible();
	await here.getByRole('button', { name: 'Follow Sari Wulandari' }).click();
	await expect(page.getByText('Following Sari Wulandari · Code · index.html')).toBeVisible();
	await expect(page.getByRole('heading', { level: 1, name: 'index.html' })).toBeVisible();
	await page.getByRole('button', { name: 'Stop following' }).click();
	await expect(page.getByText(/^Following /)).toHaveCount(0);
	await theirs.close();
});
