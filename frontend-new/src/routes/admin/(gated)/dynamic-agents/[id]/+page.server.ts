import { error, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { DynamicAgent, ToolGroup } from '$lib/admin/api';
import { agentFromForm } from '$lib/admin/agents-form';
import { act, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * A custom agent to make (`/admin/dynamic-agents/new`) or change (agents: write): what it's for,
 * how it's told to work, the tools it may use (the crew's own, from Tools), what else it can do,
 * and how its mascot looks. There's no single-agent endpoint: an existing one comes from the list.
 */
export const load: PageServerLoad = async (event) => {
	const isNew = event.params.id === 'new';
	const [list, tools] = await Promise.all([
		isNew ? null : getJson<DynamicAgent[]>(event, '/api/admin/dynamic-agents'),
		getJson<{ groups: ToolGroup[] }>(event, '/api/admin/tools'),
	]);
	// What a custom agent can be given: Tools' "Custom agents" group, the backend's catalog of them
	// (the tools the crew shares aren't in it, and the backend refuses them).
	const groups = tools.ok ? tools.data.groups.filter((group) => group.kind === 'custom').flatMap((group) => group.tools.map((tool) => tool.name)) : [];
	if (!list) return { title: m.admin_agent_new(), agent: null, tools: groups };
	const loaded = mustLoad(list);
	if (!loaded.ok) return { title: m.admin_page_dynamic_agents(), agent: loaded, tools: groups };
	const agent = loaded.data.find((one) => one.id === event.params.id);
	if (!agent) error(404, { message: m.admin_agent_missing() });
	return { title: agent.name, agent: { ok: true as const, data: agent }, tools: groups };
};

export const actions: Actions = {
	save: async (event) => {
		const read = agentFromForm(await event.request.formData());
		if (!read.ok) return refuse('save', read.message);
		if (event.params.id !== 'new') return act<DynamicAgent>(event, 'save', `/api/admin/dynamic-agents/${encodeURIComponent(event.params.id)}`, 'PUT', read.body);
		const created = await act(event, 'save', '/api/admin/dynamic-agents', 'POST', read.body);
		if ('ok' in created && created.ok) redirect(303, '/admin/dynamic-agents');
		return created;
	},
};
