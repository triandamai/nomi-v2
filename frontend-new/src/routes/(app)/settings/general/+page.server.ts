import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { DeviceItem } from '$lib/adapters/api';
import { loadGeneral, signOutDevices, startGoogleLink, unlinkGoogle } from '$lib/server/general';
import { savePreferences as saveEmails } from '$lib/server/inbox';
import { apiFetch } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';

/**
 * Settings › General (App / Settings / General m7lvY1): how the person signs in, their language,
 * zone and theme, which emails they get, and their devices. Every change saves as it's made
 * (`autosave` puts that in the top bar); the theme, language and zone through /preferences.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:general');
	const data = await loadGeneral(event);
	if (!data) error(503, { message: m.general_load_failed() });
	return { title: m.settings_general(), autosave: true, ...data };
};

export const actions: Actions = {
	/** Off to Google; it comes back to this page through /auth/google/callback. */
	linkGoogle: async (event) => {
		const started = await startGoogleLink(event, event.url.origin);
		if (!started.ok) return fail(started.status, { message: started.message });
		redirect(303, started.data.url);
	},
	unlinkGoogle: async (event) => {
		const done = await unlinkGoogle(event);
		return done.ok ? { unlinked: true } : fail(done.status, { message: done.message });
	},
	/** Every device but this one (the backend marks it `current` from this device's id). */
	signOutOthers: async (event) => {
		const response = await apiFetch(event, '/api/devices');
		if (!response.ok) return fail(response.status, { message: m.action_failed() });
		const others = ((await response.json()) as DeviceItem[]).filter((device) => !device.current).map((device) => device.id);
		const done = await signOutDevices(event, others);
		return done.ok ? { signedOut: others.length } : fail(done.status, { message: done.message });
	},
	emails: async (event) => {
		const form = await event.request.formData();
		const done = await saveEmails(event, { email_account: form.get('email_account') === 'true', email_promos: form.get('email_promos') === 'true' });
		return done.ok ? { emails: done.data } : fail(done.status, { message: done.message });
	},
};
