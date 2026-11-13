import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { confirmMemory, editMemory, forgetMemory, loadMemories } from '$lib/server/memory';
import { memoryCounts } from '$lib/settings/memory';
import type { Actions, PageServerLoad } from './$types';

/** Settings › Memory & context (App / Settings / Memory hANTE). */
export const load: PageServerLoad = async (event) => {
	event.depends('app:memory');
	const memories = await loadMemories(event);
	if (!memories) error(503, { message: m.memory_load_failed() });
	const counts = memoryCounts(memories);
	return {
		title: m.settings_memory(),
		meta: m.memory_meta({ memories: counts.all, topics: counts.topics }),
		exportLink: { href: '/settings/memory/memories.json', label: m.memory_export() },
		memories,
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
	forget: async (event) => {
		const done = await forgetMemory(event, field(await event.request.formData(), 'id'));
		return done.ok ? { forgotten: true } : fail(done.status, { message: done.message });
	},
};
