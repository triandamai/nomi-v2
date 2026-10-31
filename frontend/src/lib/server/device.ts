import type { Cookies } from '@sveltejs/kit';
import { DEVICE_COOKIE, DEVICE_COOKIE_MAX_AGE, describeBrowser } from '$lib/device';
import { APP_VERSION } from '$lib/version';
import { apiFetch } from '$lib/server/api';

/** Set once this browser is in the device list for this version; it registers again after a day. */
const REGISTERED_COOKIE = 'device_registered';
const REGISTER_EVERY_SECONDS = 24 * 60 * 60;
const PLATFORM_WEB = 'web';
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** This browser's device id, made on its first visit and kept for good. */
export function ensureDeviceId(cookies: Cookies): string {
	const saved = cookies.get(DEVICE_COOKIE);
	if (saved && UUID.test(saved)) return saved;
	const made = crypto.randomUUID();
	cookies.set(DEVICE_COOKIE, made, { path: '/', sameSite: 'lax', httpOnly: true, maxAge: DEVICE_COOKIE_MAX_AGE });
	return made;
}

/** Puts this browser in the person's device list (at most once a day, and again on a new version). */
export async function registerDevice(fetchFn: typeof fetch, cookies: Cookies, userAgent: string): Promise<void> {
	const deviceId = cookies.get(DEVICE_COOKIE);
	if (!deviceId || cookies.get(REGISTERED_COOKIE) === APP_VERSION) return;
	const response = await apiFetch(fetchFn, cookies, '/api/devices', {
		method: 'POST',
		body: JSON.stringify({ device_id: deviceId, platform: PLATFORM_WEB, app_version: APP_VERSION, ...describeBrowser(userAgent) }),
	}).catch(() => null);
	if (response?.ok) cookies.set(REGISTERED_COOKIE, APP_VERSION, { path: '/', sameSite: 'lax', httpOnly: true, maxAge: REGISTER_EVERY_SECONDS });
}
