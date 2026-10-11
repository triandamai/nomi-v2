import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { HealthConnection } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const response = await apiFetch(fetch, cookies, '/api/connections/hevy');
	const body = response.ok ? ((await response.json()) as { connection: HealthConnection | null }) : null;
	return {
		loaded: body !== null,
		connection: body?.connection ?? null,
		// The chat whose request needed Hevy: offered as the way back once connected.
		resume: url.searchParams.get('resume'),
		keyRejected: url.searchParams.get('reason') === 'key_rejected',
	};
};

export const actions: Actions = {
	connect: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const apiKey = String(data.get('api_key') ?? '').trim();
		if (!apiKey) return fail(400, { error: 'missing_key' });
		const response = await apiFetch(fetch, cookies, '/api/connections/hevy', { method: 'PUT', body: JSON.stringify({ api_key: apiKey }) });
		if (!response.ok) {
			const body = (await response.json().catch(() => null)) as { error?: string; detail?: string | null } | null;
			return fail(response.status, { error: body?.error ?? 'server_error', detail: body?.detail ?? null });
		}
		return { connected: true };
	},
	disconnect: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/connections/hevy', { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { error: 'disconnect_failed' });
		return { disconnected: true };
	},
};
