import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const run = (name: keyof typeof actions, form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0] = () => new Response(null, { status: 204 })) => {
	const api = fakeBackend(answer);
	return { api, result: actions[name](fakeEvent({ url: 'http://localhost/settings/memory', form, fetch: api.fetch, locals: { accessToken: 'token' } }) as never) };
};

describe('Settings › Memory & context', () => {
	it('names itself with how much it holds, and offers the export', async () => {
		const api = fakeBackend(() => json({ memories: [{ id: 'm1', kind: 'fact', weight: 1 }, { id: 'm2', kind: 'person', weight: 1 }] }));
		const data = await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'Memory & context', meta: '2 memories · 2 topics', exportLink: { href: '/settings/memory/memories.json' } });
	});

	it('shows the error page when memory can’t load', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('corrects a memory, but not into nothing', async () => {
		const ok = run('edit', { id: 'm1', content: ' Short answers ' });
		expect(await ok.result).toEqual({ edited: true });
		expect(ok.api.calls[0].body).toEqual({ content: 'Short answers' });
		const empty = run('edit', { id: 'm1', content: ' ' });
		expect(await empty.result).toMatchObject({ status: 400, data: { field: 'content' } });
		expect(empty.api.calls).toEqual([]);
	});

	it('confirms and forgets, passing on why the backend said no', async () => {
		expect(await run('confirm', { id: 'm1' }).result).toEqual({ confirmed: true });
		expect(await run('forget', { id: 'm1' }).result).toEqual({ forgotten: true });
		expect(await run('forget', { id: 'gone' }, () => new Response('memory not found', { status: 404 })).result).toMatchObject({ status: 404, data: { message: 'memory not found' } });
	});

	it('loads the settings and the profile beside the memories, and still opens without them', async () => {
		const settings = { learn_from_chats: true, keep_profile: true, search_past_chats: true, memory_budget: 8000, profile_budget: 1000, memory_used: 0, profile_used: 0 };
		const answer = (call: { url: string }) =>
			call.url.endsWith('/settings') ? json(settings) : call.url.endsWith('/profile') ? json({ text: '', tokens: 0, updated_at: null }) : json({ memories: [] });
		expect(await load(fakeEvent({ fetch: fakeBackend(answer).fetch, locals: { accessToken: 'token' } }) as never)).toMatchObject({ settings, profile: { text: '' } });
		const without = (call: { url: string }) => (call.url.endsWith('/memory') ? json({ memories: [] }) : new Response('', { status: 500 }));
		expect(await load(fakeEvent({ fetch: fakeBackend(without).fetch, locals: { accessToken: 'token' } }) as never)).toMatchObject({ settings: null, profile: null });
	});

	it('saves a switch or a budget, refusing what isn’t one', async () => {
		const saved = { learn_from_chats: false, keep_profile: true, search_past_chats: true, memory_budget: 8000, profile_budget: 1000, memory_used: 0, profile_used: 0 };
		const ok = run('settings', { learn_from_chats: 'false' }, () => json(saved));
		expect(await ok.result).toEqual({ settings: saved });
		expect(ok.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/memory/settings', body: { learn_from_chats: false } });
		const budget = run('settings', { memory_budget: '2200' }, () => json(saved));
		await budget.result;
		expect(budget.api.calls[0].body).toEqual({ memory_budget: 2200 });
		const bad = run('settings', { memory_budget: 'lots' });
		expect(await bad.result).toMatchObject({ status: 400 });
		expect(bad.api.calls).toEqual([]);
		expect(await run('settings', { profile_budget: '50' }, () => json({ error: 'invalid_budget' }, 400)).result).toMatchObject({ status: 400, data: { message: 'That budget is out of range.' } });
	});

	it('saves the profile in the person’s words, or says it’s too long', async () => {
		const ok = run('profile', { text: ' Mario, Jakarta. ' }, () => json({ text: 'Mario, Jakarta.', tokens: 4, updated_at: '2026-11-15T00:00:00Z' }));
		expect(await ok.result).toEqual({ profile: { text: 'Mario, Jakarta.', tokens: 4, updated_at: '2026-11-15T00:00:00Z' } });
		expect(ok.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/memory/profile', body: { text: 'Mario, Jakarta.' } });
		expect(await run('profile', { text: 'x' }, () => json({ error: 'too_long' }, 400)).result).toMatchObject({ status: 400, data: { field: 'profile', message: 'That’s longer than the profile budget. Shorten it, or raise the budget.' } });
	});
});

