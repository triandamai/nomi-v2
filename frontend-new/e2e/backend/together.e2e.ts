import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';

// Building together against the real backend (Flow / Build — Invite & build together TnwQT,
// steps 5, 7, 8): two people in one project see each other in the bar, where each is, and one
// follows the other's view until they stop or move on their own.

type Orgs = { orgs: { id: string }[] };

test('two people in one project: who’s here, where, and following', async ({ page, context, browser }) => {
	const mario = await createAccount();
	const sari = await createAccount();
	await api(mario, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Mario Prasetyo' }) });
	await api(sari, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Sari Wulandari' }) });
	const org = ((await (await api(mario, '/api/orgs')).json()) as Orgs).orgs[0].id;
	const invite = (await (await api(mario, `/api/orgs/${org}/invites`, { method: 'POST', body: JSON.stringify({ email: sari.email, role: 'member' }) })).json()) as { code: string };
	expect((await api(sari, `/api/invites/${invite.code}/accept`, { method: 'POST' })).ok).toBe(true);
	const id = ((await (await api(mario, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as { project_id: string }).project_id;
	const files = [
		{ path: 'index.html', content: '<h1>Studio Rupa</h1>' },
		{ path: 'about.html', content: '<h1>About</h1>' },
	];
	expect((await api(mario, `/api/projects/${id}/import/files`, { method: 'POST', body: JSON.stringify({ files, finish: true }) })).ok).toBe(true);

	await signInAs(context, mario);
	await page.goto(`/projects/${id}`, { waitUntil: 'networkidle' });
	const theirs = await browser.newContext();
	await signInAs(theirs, sari);
	const sariPage = await theirs.newPage();
	await sariPage.goto(`/projects/${id}`, { waitUntil: 'networkidle' });

	// Both faces in the bar; the menu says where Sari is.
	const here = page.getByRole('button', { name: 'People here: Sari Wulandari' });
	await expect(here).toBeVisible();
	await here.click();
	await expect(page.getByText('In this project · 2')).toBeVisible();
	await expect(page.getByText('Mario Prasetyo (you)')).toBeVisible();
	await sariPage.getByRole('radio', { name: 'Code' }).click();
	await expect(page.getByText('Code · index.html')).toBeVisible();

	// Following mirrors her view, file by file, until it's stopped.
	await page.getByRole('button', { name: 'Follow' }).click();
	await expect(page.getByText('Following Sari Wulandari · Code · index.html')).toBeVisible();
	await expect(page.getByRole('radio', { name: 'Code' })).toHaveAttribute('aria-checked', 'true');
	await sariPage.getByRole('treeitem', { name: /about\.html/ }).click();
	await expect(page.getByText('Following Sari Wulandari · Code · about.html')).toBeVisible();
	await expect(page.getByText('<h1>About</h1>')).toBeVisible();
	await page.getByRole('button', { name: 'Stop following' }).click();
	await expect(page.getByText(/^Following /)).toHaveCount(0);

	// Moving on your own stops following too.
	await here.click();
	await page.getByRole('button', { name: 'Follow' }).click();
	await expect(page.getByText(/^Following Sari/)).toBeVisible();
	await page.getByRole('radio', { name: 'Preview' }).click();
	await expect(page.getByText(/^Following /)).toHaveCount(0);

	// When she leaves, the bar shows only Mario again.
	await theirs.close();
	await expect(here).toHaveCount(0);
});
