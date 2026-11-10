import { describe, expect, it } from 'vitest';
import { clientHeaders } from './clientHeaders';
import { APP_VERSION } from '$lib/version';

describe('clientHeaders', () => {
	it('names the frontend version and this browser on every call to the backend', () => {
		const headers = clientHeaders(new Headers({ authorization: 'Bearer a1' }), '0f3c2a1e-5b6d-4c7e-8f90-a1b2c3d4e5f6');
		expect(Object.fromEntries(headers)).toEqual({ authorization: 'Bearer a1', 'x-client-version': APP_VERSION, 'x-device-id': '0f3c2a1e-5b6d-4c7e-8f90-a1b2c3d4e5f6' });
	});

	it('leaves the device out when the browser has none yet', () => {
		expect(clientHeaders(new Headers(), undefined).has('x-device-id')).toBe(false);
	});
});
