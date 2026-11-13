import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { crewMeta } from '$lib/settings/crew';
import { loadCrew, restorePersonality } from '$lib/server/crew';
import type { Actions, PageServerLoad } from './$types';

/**
 * Settings › Crew (App / Settings / Crew vVUeh). The member shown comes from `?member=` on the page
 * itself, so choosing one doesn't load the crew again. Nomi's and Koda's models are set through
 * Models & providers' actions.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:crew');
	const data = await loadCrew(event);
	if (!data.members) error(503, { message: m.crew_load_failed() });
	return { title: m.settings_crew(), meta: crewMeta(data.members), ...data, members: data.members };
};

export const actions: Actions = {
	/** Brings back an earlier version of the personality Nomi speaks with. */
	restore: async (event) => {
		const version = Number((await event.request.formData()).get('version'));
		if (!Number.isInteger(version) || version < 1) return fail(400, { message: m.action_failed() });
		const sent = await restorePersonality(event, version);
		return sent.ok ? { restored: version } : fail(sent.status, { message: sent.status === 404 ? m.crew_version_gone() : sent.message || m.action_failed() });
	},
};
