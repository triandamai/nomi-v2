// The drawer's pins after a change in this tab, so the drawer, the More page and Preferences all
// update at once while the save goes through. Only ever set in the browser (by savePins), so a
// server render always uses the person's saved preferences, never another request's state.

import { deserialize } from '$app/forms';
import { invalidateAll } from '$app/navigation';
import { resolvePins, type PinnableId } from '$lib/drawerPins';

const local = $state<{ pins: PinnableId[] | null }>({ pins: null });

/** The pins to show: the change just made here, else what's saved. */
export function currentPins(saved: string[] | null | undefined): PinnableId[] {
	return local.pins ?? resolvePins(saved);
}

/** Shows `pins` straight away and saves them; on failure puts the saved pins back. */
export async function savePins(pins: PinnableId[]): Promise<boolean> {
	local.pins = pins;
	const body = new FormData();
	body.set('pins', JSON.stringify(pins));
	let saved = false;
	try {
		const response = await fetch('/preferences?/updateDrawer', { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		saved = deserialize(await response.text()).type === 'success';
	} catch {
		saved = false;
	}
	if (saved) await invalidateAll();
	local.pins = null;
	return saved;
}
