import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json, settle } from '../../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { GET } = await import('./+server');

const get = () => GET({ params: { sessionId: 's1' }, cookies: cookieJar(), fetch } as never) as Promise<Response>;

beforeEach(() => apiFetch.mockReset());

describe('design studio: what there is now', () => {
	it('gives the chat’s previews, feedback and approval, for a project page to show in place', async () => {
		apiFetch.mockResolvedValue(json({ previews: [{ version: 1 }], feedback: [], approval: null }));
		const response = await get();
		expect(await response.json()).toEqual({ directions: null, previews: [{ version: 1 }], feedback: [], approval: null });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/design');
	});

	it('passes on a refusal', async () => {
		apiFetch.mockResolvedValue(json({}, 404));
		const outcome = (await settle(() => get())) as { thrown?: { status?: number } };
		expect(outcome.thrown?.status).toBe(404);
	});
});
