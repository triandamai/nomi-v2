import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadProfile, saveProfile } from '$lib/server/profile';
import { usernameProblem } from '$lib/settings/profile';
import type { Actions, PageServerLoad } from './$types';

/** How many live projects a profile can feature (App / Settings / Profile → Showcase "1 of 3"). */
const SHOWCASE_MAX = 3;

/** Settings › Profile (App / Settings / Profile U9Gnrg): what they've done, and their public profile. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:profile');
	const data = await loadProfile(event);
	if (!data) error(503, { message: m.profile_load_failed() });
	return { title: m.settings_profile(), ...data };
};

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();

function showcase(raw: string): string[] | null {
	try {
		const ids = JSON.parse(raw || '[]') as unknown;
		return Array.isArray(ids) && ids.every((id) => typeof id === 'string') && ids.length <= SHOWCASE_MAX ? ids : null;
	} catch {
		return null;
	}
}

export const actions: Actions = {
	save: async (event) => {
		const form = await event.request.formData();
		const username = field(form, 'username').replace(/^@/, '');
		const problem = usernameProblem(username);
		if (problem) return fail(400, { field: 'username', message: problem });
		const ids = showcase(field(form, 'showcase'));
		if (!ids) return fail(400, { field: 'showcase', message: m.profile_showcase_max() });
		const done = await saveProfile(event, {
			display_name: field(form, 'display_name'),
			username,
			show_stats_publicly: field(form, 'show_stats_publicly') === 'true',
			show_crew_publicly: field(form, 'show_crew_publicly') === 'true',
			showcase_project_ids: ids,
		});
		if (done.ok) return { saved: true };
		if (done.status === 409) return fail(409, { field: 'username', message: m.profile_username_taken() });
		return fail(done.status, { message: done.message });
	},
	removePhoto: async (event) => {
		const done = await saveProfile(event, { avatar_url: '' });
		return done.ok ? { photoRemoved: true } : fail(done.status, { message: done.message });
	},
};
