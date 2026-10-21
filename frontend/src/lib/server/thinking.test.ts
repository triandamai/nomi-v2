import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { loadThinkingLevel, saveThinkingLevel } = await import('./thinking');

const cookies = {} as Cookies;
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
const form = (level: string) => {
	const data = new FormData();
	data.set('level', level);
	return data;
};

beforeEach(() => apiFetch.mockReset());

describe('loadThinkingLevel', () => {
	it.each(['low', 'medium', 'high', 'max'])('reads %s', async (level) => {
		apiFetch.mockResolvedValue(json({ level }));
		expect(await loadThinkingLevel(fetch, cookies, 's1')).toBe(level);
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/thinking');
	});

	it('hides the picker for a level it doesn’t know, such as the retired "off"', async () => {
		apiFetch.mockResolvedValue(json({ level: 'off' }));
		expect(await loadThinkingLevel(fetch, cookies, 's1')).toBeNull();
	});
});

describe('saveThinkingLevel', () => {
	it.each(['low', 'medium', 'high', 'max'])('saves %s', async (level) => {
		apiFetch.mockResolvedValue(json({ level }));
		expect(await saveThinkingLevel(fetch, cookies, 's1', form(level))).toEqual({ thinkingLevel: level });
		expect(apiFetch.mock.calls[0][3]).toMatchObject({ method: 'PUT', body: JSON.stringify({ level }) });
	});

	it('refuses "off" without calling the API: the backend no longer accepts it', async () => {
		const result = await saveThinkingLevel(fetch, cookies, 's1', form('off'));
		expect(result).toMatchObject({ status: 400 });
		expect(apiFetch).not.toHaveBeenCalled();
	});
});
