import { CLIENT_VERSION_HEADER, APP_VERSION } from '$lib/version';
import { DEVICE_ID_HEADER } from '$lib/device';

/** Every call to the backend says which version of the frontend made it, and from which browser. */
export function clientHeaders(headers: Headers, deviceId: string | undefined): Headers {
	const next = new Headers(headers);
	next.set(CLIENT_VERSION_HEADER, APP_VERSION);
	if (deviceId) next.set(DEVICE_ID_HEADER, deviceId);
	return next;
}
