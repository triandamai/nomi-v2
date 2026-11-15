import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { newMemberFromForm, toolsFromForm } from '$lib/crewSettings';
import { catalogFrom } from '$lib/toolPicker';
import type { CrewTemplate } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

/** A member of the person's own: from scratch or from a template (GET /api/agents/templates). */
export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [response, catalog] = await Promise.all([apiFetch(fetch, cookies, '/api/agents/templates'), apiFetch(fetch, cookies, '/api/tools/catalog')]);
	const templates = response.ok ? ((await response.json()) as { templates: CrewTemplate[] }).templates : [];
	return { templates, catalog: catalog.ok ? catalogFrom(await catalog.json()) : [] };
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const parsed = newMemberFromForm(data);
		const values = {
			name: String(data.get('name') ?? ''),
			role: String(data.get('role') ?? ''),
			soul: String(data.get('soul') ?? ''),
			template: String(data.get('template') ?? ''),
			tools: toolsFromForm(data),
		};
		if ('invalid' in parsed) return fail(400, { invalid: parsed.invalid, ...values });
		const response = await apiFetch(fetch, cookies, '/api/agents', { method: 'POST', body: JSON.stringify(parsed.body) });
		if (!response.ok) return fail(response.status, { failed: response.status === 409 ? ('taken' as const) : ('failed' as const), ...values });
		const made = (await response.json()) as { agent_type: string };
		throw redirect(303, `/crew/${encodeURIComponent(made.agent_type)}`);
	},
};
