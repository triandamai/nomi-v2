import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { DynamicAgent } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/dynamic-agents');
	if (!response.ok) {
		return { agents: [] as DynamicAgent[] };
	}
	const agents = (await response.json()) as DynamicAgent[];
	return { agents };
};

function readForm(data: FormData) {
	const name = data.get('name');
	const systemPrompt = data.get('system_prompt');
	const intentLabel = data.get('intent_label');
	const intentDescription = data.get('intent_description');
	const grantedTools = data.getAll('granted_tools').filter((v): v is string => typeof v === 'string');
	return {
		name: typeof name === 'string' ? name : '',
		systemPrompt: typeof systemPrompt === 'string' ? systemPrompt : '',
		intentLabel: typeof intentLabel === 'string' ? intentLabel : '',
		intentDescription: typeof intentDescription === 'string' ? intentDescription : '',
		grantedTools,
		supportsTodos: data.get('supports_todos') === 'true',
		supportsPlans: data.get('supports_plans') === 'true',
		canDelegate: data.get('can_delegate') === 'true',
	};
}

function toBody(fields: ReturnType<typeof readForm>) {
	return JSON.stringify({
		name: fields.name,
		system_prompt: fields.systemPrompt,
		intent_label: fields.intentLabel,
		intent_description: fields.intentDescription,
		granted_tools: fields.grantedTools,
		supports_todos: fields.supportsTodos,
		supports_plans: fields.supportsPlans,
		can_delegate: fields.canDelegate,
	});
}

export const actions: Actions = {
	create: async ({ request, cookies, fetch }) => {
		const fields = readForm(await request.formData());
		if (!fields.name || !fields.systemPrompt || !fields.intentLabel || !fields.intentDescription) {
			return fail(400, { error: 'Name, system prompt, intent label, and intent description are required.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/admin/dynamic-agents', { method: 'POST', body: toBody(fields) });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to create agent.' });
		}
		return { success: true };
	},

	update: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const fields = readForm(data);
		if (typeof id !== 'string' || !fields.name || !fields.systemPrompt || !fields.intentLabel || !fields.intentDescription) {
			return fail(400, { error: 'Name, system prompt, intent label, and intent description are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/dynamic-agents/${id}`, { method: 'PUT', body: toBody(fields) });
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update agent.' });
		}
		return { success: true };
	},

	toggleActive: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		if (typeof id !== 'string') {
			return fail(400, { error: 'Invalid agent.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/dynamic-agents/${id}/toggle-active`, { method: 'POST' });
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to toggle agent.' });
		}
		return { success: true };
	},
};
