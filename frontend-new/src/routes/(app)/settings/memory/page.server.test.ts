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
});
