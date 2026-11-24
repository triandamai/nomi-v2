import { expect, test, type Page } from '../support/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { seedReply, setProjectStatus } from './support/seed';

// Projects against the real backend, past the first project (App / Projects JyWON and its flows):
// a card's one next action (Act from the card syolc: Resume once Koda stopped mid-build; no Deploy
// where this Nomi has no Shipyard, as locally), the detail panel (Detail panel pgSon: what's in the
// project, its chats, asking Koda for a change, × and Escape), the card menu (Card menu M9O0iU:
// Settings and Export open the workspace's own, Move to another workspace), and on a phone.

type Made = { session_id: string; project_id: string };
type Messages = { messages: { content: string }[] };

async function orgOf(account: Account): Promise<string> {
	const list = (await (await api(account, '/api/orgs')).json()) as { orgs: { id: string }[]; project_org_id: string | null };
	return list.project_org_id ?? list.orgs[0].id;
}

async function newProject(account: Account, org: string, files: { path: string; content: string }[] = []): Promise<Made> {
	const made = (await (await api(account, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as Made;
	if (files.length) expect((await api(account, `/api/projects/${made.project_id}/import/files`, { method: 'POST', body: JSON.stringify({ files, finish: true }) })).ok).toBe(true);
	return made;
}

async function said(account: Account, sessionId: string): Promise<string[]> {
	return ((await (await api(account, `/api/sessions/${sessionId}/messages`)).json()) as Messages).messages.map((message) => message.content);
}

const cardOf = (page: Page, id: string) => page.locator(`[data-project="${id}"]`).getByRole('article');

test('a card’s next action, the detail panel and the card menu', async ({ page, context }) => {
	await page.setViewportSize({ width: 1512, height: 862 });
	const account = await createAccount();
	await signInAs(context, account);
	const org = await orgOf(account);
	// One Koda left mid-build (files written, never reported done), one that's only a draft.
	const stopped = await newProject(account, org, [{ path: 'index.html', content: '<h1>Warung Kopi Bu Siti</h1>' }]);
	await setProjectStatus(stopped.project_id, 'building');
	const draft = await newProject(account, org);

	await page.goto('/projects', { waitUntil: 'networkidle' });
	const stoppedCard = cardOf(page, stopped.project_id);
	const draftCard = cardOf(page, draft.project_id);
	await expect(stoppedCard).toContainText('Stopped');
	await expect(stoppedCard).toContainText('nothing deployed · files kept');
	await expect(draftCard).toContainText('Draft');
	await expect(draftCard).toContainText('not deployed yet');
	// No Shipyard here: no Deploy on any card; the draft has no files to deploy anyway.
	await expect(page.getByRole('button', { name: 'Deploy' })).toHaveCount(0);

	// The first project shows in the panel on a wide desktop; selecting another swaps it.
	const panel = page.getByRole('complementary', { name: 'New project' });
	await expect(panel).toBeVisible();
	await stoppedCard.getByText('nothing deployed · files kept').click();
	await expect(stoppedCard.getByRole('button', { name: 'New project', exact: true })).toHaveAttribute('aria-pressed', 'true');
	await expect(panel.getByRole('definition').first()).toHaveText('1 file · static site');
	await expect(panel.getByText('URL', { exact: true }).locator('..')).toContainText('after the first deploy');
	await expect(panel.getByText('Data', { exact: true }).locator('..')).toContainText('none yet');
	await expect(panel.getByText('Deploys', { exact: true }).locator('..')).toContainText('none yet');
	await expect(panel.getByText('Members', { exact: true }).locator('..')).toContainText('you (owner)');
	await expect(panel.getByRole('link', { name: /Building it with Koda|New project/ }).first()).toHaveAttribute('href', `/projects/${stopped.project_id}`);
	await expect(panel.getByRole('link', { name: /Open workspace/ })).toHaveAttribute('href', `/projects/${stopped.project_id}`);

	// Resume asks Koda to carry on, in the project's chat.
	await stoppedCard.getByRole('button', { name: 'Resume' }).click();
	await expect.poll(() => said(account, stopped.session_id)).toContain('Carry on building where you stopped.');

	// Asking from the panel: the change goes to Koda in the project's chat.
	await panel.getByRole('button', { name: 'Ask Koda to change something' }).click();
	const field = panel.getByRole('textbox', { name: 'Ask Koda about New project' });
	await expect(field).toBeFocused();
	await expect(panel.getByText('to Koda in New project’s chat')).toBeVisible();
	await field.fill('Add a contact page');
	await page.keyboard.press('Enter');
	await expect.poll(() => said(account, stopped.session_id)).toContain('Add a contact page');
	await expect(panel.getByRole('button', { name: 'Ask Koda to change something' })).toBeVisible();

	// × closes it; a click on a card brings it back; Escape closes it again.
	await panel.getByRole('button', { name: 'Close' }).click();
	await expect(panel).toBeHidden();
	await draftCard.getByText('not deployed yet').click();
	await expect(panel).toBeVisible();
	await expect(panel.getByRole('definition').first()).toHaveText('0 files · SvelteKit');
	await page.keyboard.press('Escape');
	await expect(panel).toBeHidden();

	// Settings opens the workspace with its Settings, and the address is tidied.
	await stoppedCard.getByRole('button', { name: 'New project actions' }).click();
	await page.getByRole('menuitem', { name: 'Settings' }).click();
	await expect(page).toHaveURL(new RegExp(`/projects/${stopped.project_id}$`));
	await expect(page.getByRole('region', { name: 'Settings' }).getByRole('heading', { name: 'Ownership and availability' })).toBeVisible();

	// Move: to another workspace the person is in, at once.
	const other = await api(account, '/api/orgs', { method: 'POST', body: JSON.stringify({ name: 'Kopi Kita' }) });
	expect(other.ok).toBe(true);
	await api(account, '/api/preferences/project-org', { method: 'PUT', body: JSON.stringify({ org_id: org }) });
	await page.goto('/projects', { waitUntil: 'networkidle' });
	await cardOf(page, draft.project_id).getByRole('button', { name: 'New project actions' }).click();
	await page.getByRole('menuitem', { name: /Move to workspace/ }).click();
	const move = page.getByRole('dialog', { name: 'Move New project to another workspace' });
	await expect(move.getByRole('radio', { name: /E2E space/ })).toHaveAttribute('aria-disabled', 'true');
	await move.getByRole('radio', { name: /Kopi Kita/ }).click();
	await expect(move.getByText(/The URL, deploys and data don’t change\./)).toBeVisible();
	await move.getByRole('button', { name: 'Move project' }).click();
	await expect(page.getByText('New project moved to Kopi Kita')).toBeVisible();
	await expect(page.getByRole('article')).toHaveCount(1);
	const moved = (await (await api(account, `/api/projects/${draft.project_id}`)).json()) as { org_name: string };
	expect(moved.org_name).toBe('Kopi Kita');
});

test('on a phone, a project opens its own screen (Mobile / Project Detail), and Export opens the workspace’s', async ({ page, context }) => {
	await page.setViewportSize({ width: 390, height: 844 });
	const account = await createAccount();
	await signInAs(context, account);
	const made = await newProject(account, await orgOf(account), [{ path: 'index.html', content: '<h1>Menu</h1>' }]);

	await page.goto('/projects', { waitUntil: 'networkidle' });
	const screen = page.getByRole('region', { name: 'New project' });
	await expect(screen).toBeHidden();
	const card = page.getByRole('article', { name: 'New project' });
	await card.getByRole('button', { name: 'New project', exact: true }).click();
	await expect(screen.getByRole('heading', { level: 1, name: 'New project' })).toBeVisible();
	await expect(screen.getByRole('link', { name: 'Open workspace' })).toHaveAttribute('href', `/projects/${made.project_id}`);
	await expect(screen.getByText('What’s in this project')).toBeVisible();
	// A screen inside the Projects tab: no tab bar under it, as the pen draws it.
	await expect(page.getByRole('navigation', { name: 'Main' })).toHaveCount(0);
	await screen.getByRole('button', { name: 'Projects' }).click();
	await expect(screen).toBeHidden();
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Projects' })).toHaveAttribute('aria-current', 'page');

	// On a phone the card's ⋯ is a sheet (Flow / Mobile Projects — Card menu).
	await card.getByRole('button', { name: 'New project actions' }).click();
	await page.getByRole('dialog', { name: 'New project' }).getByRole('button', { name: /Export repo/ }).click();
	await expect(page).toHaveURL(new RegExp(`/projects/${made.project_id}$`));
	await expect(page.getByRole('dialog')).toBeVisible();
});

test('a project waiting on the person, renamed on its card, and copied', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	const org = await orgOf(account);
	const asked = await newProject(account, org, [{ path: 'index.html', content: '<h1>Warung</h1>' }]);
	// Koda asked in the project's chat and nobody answered yet: the card needs the person.
	await seedReply(asked.session_id, { content: 'Two questions', agent: 'Koda', blocks: [{ kind: 'questions', intro: null, questions: [] }] });
	await page.goto('/projects', { waitUntil: 'networkidle' });

	const card = cardOf(page, asked.project_id);
	await expect(card.getByText('Koda has a question')).toBeVisible();
	await page.getByRole('radio', { name: /Needs attention/ }).click();
	await expect(card).toBeVisible();
	await page.getByRole('radio', { name: /^All/ }).click();

	// Rename on the card itself.
	await card.getByRole('button', { name: 'New project actions' }).click();
	await page.getByRole('menuitem', { name: /Rename/ }).click();
	const field = card.getByRole('textbox', { name: 'Project name' });
	await field.fill('warung-kopi-menu');
	await field.press('Enter');
	await expect(page.getByText('Renamed to warung-kopi-menu')).toBeVisible();
	await expect(cardOf(page, asked.project_id).getByText('warung-kopi-menu', { exact: true })).toBeVisible();

	// Duplicate makes a copy beside it, and the toast opens it.
	await cardOf(page, asked.project_id).getByRole('button', { name: 'warung-kopi-menu actions' }).click();
	await page.getByRole('menuitem', { name: /Duplicate/ }).click();
	await expect(page.getByText('Made a copy of warung-kopi-menu')).toBeVisible();
	await expect(page.locator('[data-project]')).toHaveCount(2);

	// Answer opens the project's chat, where Koda asked.
	await cardOf(page, asked.project_id).getByRole('button', { name: 'Answer' }).click();
	await expect(page).toHaveURL(new RegExp(`/projects/${asked.project_id}$`));
});
