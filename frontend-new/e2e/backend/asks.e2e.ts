import { expect, test, type Page } from '@playwright/test';
import postgres from 'postgres';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { DATABASE_URL } from './support/env';

// Two people, one Koda, against the real backend (Flow / Build — Invite & build together TnwQT):
// asks in line in the project's chat, one change at a time, with a teammate's ask let go first
// (step 9); and someone who can't publish asking the project's admin to, who publishes or
// declines while the one who asked can take it back (step 10). This Nomi has no Shipyard, so
// publishing a request can't start: the admin is told why and the request stays open.

type Orgs = { orgs: { id: string }[] };
type Line = { asks: { id: string; state: string }[] };

/** Mario (admin of the project, as its maker) and Sari (a member of his workspace, so she can prompt but not publish). */
async function together() {
	const mario = await createAccount();
	const sari = await createAccount();
	await api(mario, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Mario Prasetyo' }) });
	await api(sari, '/api/profile', { method: 'PUT', body: JSON.stringify({ display_name: 'Sari Wulandari' }) });
	const org = ((await (await api(mario, '/api/orgs')).json()) as Orgs).orgs[0].id;
	const invite = (await (await api(mario, `/api/orgs/${org}/invites`, { method: 'POST', body: JSON.stringify({ email: sari.email, role: 'member' }) })).json()) as { code: string };
	expect((await api(sari, `/api/invites/${invite.code}/accept`, { method: 'POST' })).ok).toBe(true);
	const id = ((await (await api(mario, '/api/projects', { method: 'POST', body: JSON.stringify({ org_id: org }) })).json()) as { project_id: string }).project_id;
	const files = [{ path: 'index.html', content: '<h1>Warung Kopi Bu Siti</h1>' }];
	expect((await api(mario, `/api/projects/${id}/import/files`, { method: 'POST', body: JSON.stringify({ files, finish: true }) })).ok).toBe(true);
	const session = ((await (await api(mario, `/api/projects/${id}`)).json()) as { session_id: string }).session_id;
	return { mario, sari, id, session };
}

async function line(account: Account, session: string): Promise<Line['asks']> {
	return ((await (await api(account, `/api/sessions/${session}/queue`)).json()) as Line).asks;
}

async function withDb<T>(run: (sql: postgres.Sql) => Promise<T>): Promise<T> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		return await run(sql);
	} finally {
		await sql.end();
	}
}

async function open(page: Page, context: Parameters<typeof signInAs>[0], account: Account, id: string) {
	await signInAs(context, account);
	await page.goto(`/projects/${id}`, { waitUntil: 'networkidle' });
}

test('asks wait in line in the project’s chat, and a teammate’s can go first', async ({ page, context }) => {
	const { mario, sari, id, session } = await together();
	// Koda busy with an ask of Mario's (the fake model answers at once, so the running one is put
	// in as a turn the worker has claimed): what's asked next waits.
	expect((await api(mario, `/api/sessions/${session}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Hello Koda' }) })).ok).toBe(true);
	await expect.poll(() => line(mario, session), { timeout: 30_000 }).toEqual([]);
	const running = await withDb(async (sql) => {
		const [{ sender }] = await sql`SELECT sender_channel_identity_id AS sender FROM turn_jobs WHERE session_id = ${session} LIMIT 1`;
		const [row] = await sql`INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text, status, claimed_at) VALUES (${session}, ${sender}, 'Rename the shop to Warung Kopi Bu Siti', 'processing', now()) RETURNING id`;
		return row.id as string;
	});

	await open(page, context, mario, id);
	const chat = page.getByRole('region', { name: 'Chat' });
	// One ask (running) isn't a line yet.
	await expect(chat.getByRole('region', { name: /asks? in line/ })).toHaveCount(0);

	// Mario asks from the chat; Sari asks from hers: the line shows as the chat moves on.
	await chat.getByRole('textbox').fill('Make the Pre-order button green');
	await chat.getByRole('textbox').press('Enter');
	expect((await api(sari, `/api/sessions/${session}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Add a Makanan section with nasi uduk' }) })).ok).toBe(true);
	const queue = chat.getByRole('region', { name: '3 asks in line' });
	await expect(queue).toBeVisible();
	await expect(queue.getByText('Koda is building')).toBeVisible();
	await expect(queue.getByText('Mario Prasetyo · next')).toBeVisible();
	await expect(queue.getByText('Sari Wulandari · waiting')).toBeVisible();

	// Sari's waits behind Mario's: he lets her go first.
	await queue.getByRole('button', { name: 'Do Sari’s first' }).or(queue.getByRole('button', { name: "Do Sari's first" })).click();
	await expect(queue.getByText('Sari Wulandari · next')).toBeVisible();
	await expect(queue.getByText('Mario Prasetyo · waiting')).toBeVisible();
	await expect(queue.getByRole('button', { name: /first/ })).toHaveCount(0);
	expect((await line(mario, session)).map((ask) => ask.state)).toEqual(['running', 'next', 'queued']);

	// Koda finishes the running ask, then the rest in turn: the line empties and goes away.
	await withDb((sql) => sql`UPDATE turn_jobs SET status = 'completed', completed_at = now() WHERE id = ${running}`);
	await expect(chat.getByRole('region', { name: /asks? in line/ })).toHaveCount(0, { timeout: 30_000 });
});

test('someone who can’t publish asks to; the admin is told why a publish can’t start, or declines, and she can take it back', async ({ page, context, browser }) => {
	const { mario, sari, id } = await together();
	await open(page, context, mario, id);
	const theirs = await browser.newContext();
	const sariPage = await theirs.newPage();
	await open(sariPage, theirs, sari, id);

	// Sari asks where Publish would be; Mario's chat shows her request as it arrives.
	await sariPage.getByRole('button', { name: 'Ask to publish' }).click();
	await expect(sariPage.getByText('Asked the project’s admins to publish')).toBeVisible();
	const asked = sariPage.getByRole('button', { name: 'Asked to publish' });
	await expect(asked).toBeVisible();
	// The request's own card (the innermost section saying so).
	const card = page.locator('section').filter({ hasText: /Sari.s change is ready to publish/, hasNot: page.locator('section') });
	await expect(card).toBeVisible();
	await expect(card.getByText(/asked \d\d:\d\d/)).toBeVisible();
	// An admin publishes themselves: no asking for Mario.
	await expect(page.getByRole('button', { name: 'Ask to publish' })).toHaveCount(0);

	// Publishing it can't start on this Nomi: Mario is told the backend's reason and it stays open.
	await card.getByRole('button', { name: 'Publish' }).click();
	await expect(page.getByText('That didn’t start')).toBeVisible();
	await expect(page.getByText(/publishing isn.t set up/i)).toBeVisible();
	await expect(card).toBeVisible();

	// Review in preview takes him to the preview.
	await card.getByRole('button', { name: 'Review in preview' }).click();
	await expect(page.getByRole('radio', { name: 'Preview' })).toHaveAttribute('aria-checked', 'true');

	// Sari takes it back: the card goes from Mario's chat.
	await asked.click();
	await expect(sariPage.getByRole('dialog', { name: 'You asked to publish' })).toBeVisible();
	await sariPage.getByRole('button', { name: 'Take it back' }).click();
	await expect(sariPage.getByText('Request taken back')).toBeVisible();
	await expect(sariPage.getByRole('button', { name: 'Ask to publish' })).toBeVisible();
	await expect(card).toHaveCount(0);

	// She asks again; Mario declines, and she hears so.
	await sariPage.getByRole('button', { name: 'Ask to publish' }).click();
	await expect(card).toBeVisible();
	await page.getByRole('button', { name: 'Decline Sari Wulandari’s request' }).click();
	await expect(page.getByText('Declined Sari’s request')).toBeVisible();
	await expect(card).toHaveCount(0);
	await expect(sariPage.getByText('Mario Prasetyo declined to publish your change')).toBeVisible();
	await expect(sariPage.getByRole('button', { name: 'Ask to publish' })).toBeVisible();
	await theirs.close();
});
