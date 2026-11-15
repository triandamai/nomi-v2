import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, settle } from '../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { load } = await import('./+page.server');

const preview = (version: number) => ({ version, direction_id: 'editorial', title: `Pass ${version}`, pages: [{ path: 'index.html', title: 'Home' }], elements: [], preview_token: 'ab'.repeat(32), published_url: null, created_at: '2026-11-15T10:00:00Z' });
const studio = (url = 'http://localhost/chat/s1/design') => event({ url, params: { sessionId: 's1' } });

beforeEach(() => apiFetch.mockReset());

describe('design studio: load', () => {
	it('loads the chat’s previews and the version the link asked for', async () => {
		apiFetch.mockResolvedValue(json({ previews: [preview(1), preview(2)] }));
		const data = (await load(studio('http://localhost/chat/s1/design?version=1'))) as Record<string, any>;
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/design');
		expect(data.previews.map((p: { version: number }) => p.version)).toEqual([1, 2]);
		expect(data.wanted).toBe('1');
		expect(data.sessionId).toBe('s1');
	});

	it('opens with nothing asked for', async () => {
		apiFetch.mockResolvedValue(json({ previews: [] }));
		const data = (await load(studio())) as Record<string, any>;
		expect(data.previews).toEqual([]);
		expect(data.wanted).toBeNull();
	});

	it('is not there where Design is not rolled out, or for someone else’s chat', async () => {
		apiFetch.mockResolvedValue(json({}, 404));
		const outcome = (await settle(() => load(studio()))) as { thrown?: { status?: number } };
		expect(outcome.thrown?.status).toBe(404);
	});
});
