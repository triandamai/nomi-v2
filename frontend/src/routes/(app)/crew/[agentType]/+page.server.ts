import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { levelsFromForm, SOUL_MAX_CHARS, toolsFromForm, TOOLS_SET_FIELD } from '$lib/crewSettings';
import { catalogFrom } from '$lib/toolPicker';
import type { CrewActivityItem, CrewPermission, CrewSoul, CrewSoulVersion } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

/** One crew member's settings: On, Permissions, soul.md with its versions, and Activity. */
export const load: PageServerLoad = async ({ cookies, fetch, params, parent }) => {
	const base = `/api/agents/${encodeURIComponent(params.agentType)}`;
	const read = async <T>(path: string): Promise<T | null> => {
		const response = await apiFetch(fetch, cookies, path);
		return response.ok ? ((await response.json()) as T) : null;
	};
	const [{ crew }, permissions, soul, versions, activity, catalog] = await Promise.all([
		parent(),
		read<CrewPermission[]>(`${base}/permissions`),
		read<CrewSoul>(`${base}/soul`),
		read<{ versions: CrewSoulVersion[] }>(`${base}/soul/versions`),
		read<{ items: CrewActivityItem[] }>(`${base}/activity`),
		read<unknown>('/api/tools/catalog'),
	]);
	const member = crew.find((m) => m.agent_type === params.agentType);
	if (!member || permissions === null) throw error(404, 'Not found');
	return {
		member,
		permissions,
		soul: soul ?? { text: '', version: 0 },
		versions: versions?.versions ?? [],
		activity: activity?.items ?? [],
		catalog: catalogFrom(catalog),
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
	/** Sets the tools of a member the person made. */
	tools: async ({ request, cookies, fetch, params }) => {
		const tools = toolsFromForm(await request.formData());
		const response = await apiFetch(fetch, cookies, `/api/agents/${encodeURIComponent(params.agentType)}`, {
			method: 'PUT',
			body: JSON.stringify({ tools }),
		});
		if (!response.ok) return fail(response.status, { failed: 'tools' as const });
		return { saved: 'tools' as const };
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
		const data = await request.formData();
		const text = String(data.get('text') ?? '');
		if (text.trim().length > SOUL_MAX_CHARS) return fail(400, { failed: 'soul' as const });
		const base = `/api/agents/${encodeURIComponent(params.agentType)}`;
		const response = await apiFetch(fetch, cookies, `${base}/soul`, { method: 'PUT', body: JSON.stringify({ text }) });
		if (!response.ok) return fail(response.status, { failed: 'soul' as const });
		// A tool mentioned in the soul joined the member's tools in the same form: they save together.
		if (data.has(TOOLS_SET_FIELD)) {
			const tools = await apiFetch(fetch, cookies, base, { method: 'PUT', body: JSON.stringify({ tools: toolsFromForm(data) }) });
			if (!tools.ok) return fail(tools.status, { failed: 'tools' as const });
		}
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
