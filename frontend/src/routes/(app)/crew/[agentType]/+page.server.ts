import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { levelsFromForm, SOUL_MAX_CHARS } from '$lib/crewSettings';
import type { CrewActivityItem, CrewPermission, CrewSoul, CrewSoulVersion } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

/** One crew member's settings: On, Permissions, soul.md with its versions, and Activity. */
export const load: PageServerLoad = async ({ cookies, fetch, params, parent }) => {
	const base = `/api/agents/${encodeURIComponent(params.agentType)}`;
	const read = async <T>(path: string): Promise<T | null> => {
		const response = await apiFetch(fetch, cookies, path);
		return response.ok ? ((await response.json()) as T) : null;
	};
	const [{ crew }, permissions, soul, versions, activity] = await Promise.all([
		parent(),
		read<CrewPermission[]>(`${base}/permissions`),
		read<CrewSoul>(`${base}/soul`),
		read<{ versions: CrewSoulVersion[] }>(`${base}/soul/versions`),
		read<{ items: CrewActivityItem[] }>(`${base}/activity`),
	]);
	const member = crew.find((m) => m.agent_type === params.agentType);
	if (!member || permissions === null) throw error(404, 'Not found');
	return {
		member,
		permissions,
		soul: soul ?? { text: '', version: 0 },
		versions: versions?.versions ?? [],
		activity: activity?.items ?? [],
	};
};

export const actions: Actions = {
	/** Switches the member on or off (Nomi stays on). */
	enabled: async ({ request, cookies, fetch, params }) => {
		const enabled = ['on', 'true'].includes(String((await request.formData()).get('enabled')));
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}`, {
			method: 'PUT',
			body: JSON.stringify({ enabled }),
		});
		if (!response.ok) return fail(response.status, { failed: 'enabled' as const });
		return { saved: 'enabled' as const };
	},
	permissions: async ({ request, cookies, fetch, params }) => {
		const levels = levelsFromForm(await request.formData());
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}/permissions`, {
			method: 'PUT',
			body: JSON.stringify(levels),
		});
		if (!response.ok) return fail(response.status, { failed: 'permissions' as const });
		return { saved: 'permissions' as const };
	},
	soul: async ({ request, cookies, fetch, params }) => {
		const text = String((await request.formData()).get('text') ?? '');
		if (text.trim().length > SOUL_MAX_CHARS) return fail(400, { failed: 'soul' as const });
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}/soul`, {
			method: 'PUT',
			body: JSON.stringify({ text }),
		});
		if (!response.ok) return fail(response.status, { failed: 'soul' as const });
		return { saved: 'soul' as const };
	},
	/** Puts an earlier soul.md version back in use. */
	restore: async ({ request, cookies, fetch, params }) => {
		const version = Number((await request.formData()).get('version'));
		if (!Number.isInteger(version) || version < 1) return fail(400, { failed: 'soul' as const });
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}/soul/rollback`, {
			method: 'POST',
			body: JSON.stringify({ version }),
		});
		if (!response.ok) return fail(response.status, { failed: 'soul' as const });
		return { saved: 'soul' as const };
	},
	/** Removes a member of the person's own. */
	remove: async ({ cookies, fetch, params }) => {
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}`, { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { failed: 'remove' as const });
		throw redirect(303, '/crew');
	},
};
