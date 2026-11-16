import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { confirmMemory, editMemory, forgetMemory, loadMemories, loadMemorySettings, loadProfile, saveMemorySettings, saveProfile } from '$lib/server/memory';
import type { MemorySettingsChange } from '$lib/ds/settings/MemorySettings.svelte';
import { memoryCounts } from '$lib/settings/memory';
import type { Actions, PageServerLoad } from './$types';

/** Settings › Memory & context (App / Settings / Memory hANTE). */
export const load: PageServerLoad = async (event) => {
	event.depends('app:memory');
	const [memories, settings, profile] = await Promise.all([loadMemories(event), loadMemorySettings(event), loadProfile(event)]);
	if (!memories) error(503, { message: m.memory_load_failed() });
	const counts = memoryCounts(memories);
	return {
		title: m.settings_memory(),
		meta: m.memory_meta({ memories: counts.all, topics: counts.topics }),
		exportLink: { href: '/settings/memory/memories.json', label: m.memory_export() },
		memories,
		/** Null when they couldn't be read: the topics still show. */
		settings,
		profile,
	};
};

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();

export const actions: Actions = {
	edit: async (event) => {
		const form = await event.request.formData();
		const content = field(form, 'content');
		if (!content) return fail(400, { field: 'content', message: m.memory_empty() });
		const done = await editMemory(event, field(form, 'id'), content);
		return done.ok ? { edited: true } : fail(done.status, { message: done.message });
	},
	confirm: async (event) => {
		const done = await confirmMemory(event, field(await event.request.formData(), 'id'));
		return done.ok ? { confirmed: true } : fail(done.status, { message: done.message });
	},
	/** One switch (`learn_from_chats`, `keep_profile`, `search_past_chats`: "true"/"false") or budget. */
	settings: async (event) => {
		const form = await event.request.formData();
		const change: MemorySettingsChange = {};
		for (const key of ['learn_from_chats', 'keep_profile', 'search_past_chats'] as const) {
			const value = form.get(key);
			if (value !== null) change[key] = value === 'true';
		}
		for (const key of ['memory_budget', 'profile_budget'] as const) {
			const value = form.get(key);
			if (value === null) continue;
			const budget = Number(value);
			if (!Number.isInteger(budget)) return fail(400, { message: m.memory_budget_invalid() });
			change[key] = budget;
		}
		if (!Object.keys(change).length) return fail(400, { message: m.action_failed() });
		const done = await saveMemorySettings(event, change);
		if (done.ok) return { settings: done.data };
		return fail(done.status, { message: done.code === 'invalid_budget' ? m.memory_budget_invalid() : done.message || m.action_failed() });
	},
	profile: async (event) => {
		const done = await saveProfile(event, field(await event.request.formData(), 'text'));
		if (done.ok) return { profile: done.data };
		return fail(done.status, { field: 'profile', message: done.code === 'too_long' ? m.memory_profile_too_long() : done.message || m.action_failed() });
	},
	forget: async (event) => {
		const done = await forgetMemory(event, field(await event.request.formData(), 'id'));
		return done.ok ? { forgotten: true } : fail(done.status, { message: done.message });
	},
};
