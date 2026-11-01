import { connectUserActivity } from '$lib/userActivity';

/**
 * Keeps the person's live feed open for as long as the component that calls it, on one socket
 * per device. Every invalidateAll() hands the layout a new `data`; reconnecting on each would
 * drop a socket that's still connecting and blink this tab's presence.
 */
export function useUserActivity(deviceId: () => string | null) {
	const id = $derived(deviceId());
	$effect(() => connectUserActivity(id));
}
