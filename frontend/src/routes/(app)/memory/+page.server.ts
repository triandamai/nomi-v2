import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MemoryListResponse, MemoryProfile, MemorySettings } from '$lib/types';
import { MEMORY_BUDGET_RANGE, PROFILE_BUDGET_RANGE, settingsFromForm } from '$lib/memorySettings';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [response, settingsResponse, profileResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/memory'),
		apiFetch(fetch, cookies, '/api/memory/settings'),
		apiFetch(fetch, cookies, '/api/memory/profile'),
	]);
	const { memories } = response.ok ? ((await response.json()) as MemoryListResponse) : { memories: [] };
	const settings = settingsResponse.ok ? ((await settingsResponse.json()) as MemorySettings) : null;
	const profile = profileResponse.ok ? ((await profileResponse.json()) as MemoryProfile) : null;
	return { memories, loadFailed: !response.ok, settings, profile };
};

export const actions: Actions = {
	/** The switches and budgets of Settings → Memory. */
	settings: async ({ request, cookies, fetch }) => {
		const parsed = settingsFromForm(await request.formData());
		if ('invalid' in parsed) {
			const range = parsed.invalid === 'memory_budget' ? MEMORY_BUDGET_RANGE : PROFILE_BUDGET_RANGE;
			return fail(400, { settingsError: m.mem_set_invalid_budget({ min: range.min, max: range.max }) });
		}
		const response = await apiFetch(fetch, cookies, '/api/memory/settings', { method: 'PUT', body: JSON.stringify(parsed.update) });
		if (!response.ok) return fail(response.status, { settingsError: m.mem_set_failed() });
		return { settingsSaved: true };
	},
	/** The person's own wording of their profile. */
	profile: async ({ request, cookies, fetch }) => {
		const text = String((await request.formData()).get('text') ?? '');
		const response = await apiFetch(fetch, cookies, '/api/memory/profile', { method: 'PUT', body: JSON.stringify({ text }) });
		if (!response.ok) {
			const code = ((await response.json().catch(() => ({}))) as { error?: string }).error;
			return fail(response.status, { profileError: code === 'too_long' ? m.mem_profile_too_long() : m.mem_profile_failed(), text });
		}
		return { profileSaved: true };
	},
	forget: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !id) {
			return fail(400, { error: m.mem_pick() });
		}
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) {
			return fail(response.status, { error: m.mem_forget_failed() });
		}
		return { forgotten: id };
	},
	/** The person rewrote a memory (and maybe its kind). */
	edit: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const content = String(data.get('content') ?? '').trim();
		const kind = data.get('kind');
		if (typeof id !== 'string' || !id) return fail(400, { error: m.mem_pick() });
		if (!content) return fail(400, { error: m.mem_edit_empty() });
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}`, {
			method: 'PUT',
			body: JSON.stringify({ content, kind: typeof kind === 'string' && kind ? kind : null }),
		});
		if (!response.ok) return fail(response.status, { error: m.mem_edit_failed() });
		return { edited: id };
	},
	/** The person says a memory is still true. */
	confirm: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !id) return fail(400, { error: m.mem_pick() });
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}/confirm`, { method: 'POST' });
		if (!response.ok && response.status !== 404) return fail(response.status, { error: m.mem_confirm_failed() });
		return { confirmed: id };
	},
};
