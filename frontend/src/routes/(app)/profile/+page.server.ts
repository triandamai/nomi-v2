import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Profile, ProjectSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [profileResponse, projectsResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/projects').catch(() => null),
	]);
	const profile: Profile | null = profileResponse.ok ? ((await profileResponse.json()) as Profile) : null;
	const projects: ProjectSummary[] = projectsResponse?.ok
		? ((await projectsResponse.json()) as { projects: ProjectSummary[] }).projects
		: [];
	return { profile, projects };
};

export const actions: Actions = {
	updateProfile: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const displayName = data.get('display_name');
		const username = data.get('username');
		const avatarUrl = data.get('avatar_url');
		const location = data.get('location');
		const showStats = data.get('show_stats_publicly');
		const showCrew = data.get('show_crew_publicly');
		const showcaseIdsRaw = data.get('showcase_project_ids');

		let showcaseProjectIds: string[] | undefined = undefined;
		if (typeof showcaseIdsRaw === 'string') {
			try {
				showcaseProjectIds = JSON.parse(showcaseIdsRaw);
			} catch {
				showcaseProjectIds = showcaseIdsRaw.split(',').filter(Boolean);
			}
		}

		const response = await apiFetch(fetch, cookies, '/api/profile', {
			method: 'PUT',
			body: JSON.stringify({
				display_name: typeof displayName === 'string' && displayName.trim().length > 0 ? displayName.trim() : null,
				username: typeof username === 'string' && username.trim().length > 0 ? username.trim().replace(/^@/, '') : null,
				avatar_url: typeof avatarUrl === 'string' ? avatarUrl : undefined,
				location: typeof location === 'string' ? location.trim() : null,
				show_stats_publicly: showStats === 'true' || showStats === 'on' || showStats === '1',
				show_crew_publicly: showCrew === 'true' || showCrew === 'on' || showCrew === '1',
				showcase_project_ids: showcaseProjectIds,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_save_profile() });
		}
		const updatedProfile = (await response.json()) as Profile;
		return { success: true, profile: updatedProfile };
	},
};
