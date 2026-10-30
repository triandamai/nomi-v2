import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { moveSession, pinSession } = await import('./updateSession');

const cookies = {} as Cookies;
const form = (fields: Record<string, string>) => {
	const data = new FormData();
	for (const [key, value] of Object.entries(fields)) data.set(key, value);
	return data;
};
const sent = () => JSON.parse(apiFetch.mock.calls[0][3].body);

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockResolvedValue(Response.json({ title: 'Trip', pinned: true, in_project_id: null }));
});

describe('pinSession', () => {
	it('patches the chat with the pin the form asked for', async () => {
		const result = await pinSession(fetch, cookies, 's1', form({ pinned: 'true' }));
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1');
		expect(apiFetch.mock.calls[0][3].method).toBe('PATCH');
		expect(sent()).toEqual({ pinned: true });
		expect(result).toEqual({ updated: { title: 'Trip', pinned: true, in_project_id: null } });
	});

	it('unpins on anything but "true"', async () => {
		await pinSession(fetch, cookies, 's1', form({ pinned: 'false' }));
		expect(sent()).toEqual({ pinned: false });
	});

	it('says it failed when the API refuses', async () => {
		apiFetch.mockResolvedValue(new Response('nope', { status: 404 }));
		const result = await pinSession(fetch, cookies, 's1', form({ pinned: 'true' }));
		expect(result).toMatchObject({ status: 404, data: { updateError: expect.any(String) } });
	});
});

describe('moveSession', () => {
	it('moves the chat into the picked project', async () => {
		await moveSession(fetch, cookies, 's1', form({ projectId: 'p1' }));
		expect(sent()).toEqual({ in_project_id: 'p1' });
	});

	it('takes it out of its project when none is picked', async () => {
		await moveSession(fetch, cookies, 's1', form({ projectId: '' }));
		expect(sent()).toEqual({ in_project_id: null });
	});
});
