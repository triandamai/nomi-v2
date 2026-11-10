import { describe, expect, it, vi } from 'vitest';
import { handleFetch } from './hooks.server';
import { cookieJar, event } from './test-stubs/server';
import { CLIENT_VERSION_HEADER, APP_VERSION } from '$lib/version';
import { DEVICE_COOKIE, DEVICE_ID_HEADER } from '$lib/device';
import { apiUrl } from '$lib/server/api';

describe('hooks.server handleFetch', () => {
	it('injects client version and device ID on API requests', async () => {
		const cookies = cookieJar({ [DEVICE_COOKIE]: 'device-123' });
		const ev = event({ cookies });
		const mockFetch = vi.fn(async (_req: Request) => new Response('ok'));

		const req = new Request(apiUrl('/api/me'));
		await handleFetch({ event: ev, request: req, fetch: mockFetch as unknown as typeof fetch });

		expect(mockFetch).toHaveBeenCalled();
		const calledReq = mockFetch.mock.calls[0][0];
		expect(calledReq.headers.get(CLIENT_VERSION_HEADER)).toBe(APP_VERSION);
		expect(calledReq.headers.get(DEVICE_ID_HEADER)).toBe('device-123');
	});

	it('leaves non-API requests untouched', async () => {
		const cookies = cookieJar({ [DEVICE_COOKIE]: 'device-123' });
		const ev = event({ cookies });
		const mockFetch = vi.fn(async (_req: Request) => new Response('ok'));

		const req = new Request('https://example.com/asset.png');
		await handleFetch({ event: ev, request: req, fetch: mockFetch as unknown as typeof fetch });

		expect(mockFetch).toHaveBeenCalled();
		const calledReq = mockFetch.mock.calls[0][0];
		expect(calledReq.headers.get(CLIENT_VERSION_HEADER)).toBeNull();
		expect(calledReq.headers.get(DEVICE_ID_HEADER)).toBeNull();
	});
});
