import { fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { Configuration, TestResult } from '$lib/admin/api';
import { getJson, mustLoad, refuse, send } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Configuration (configuration: view; manage changes it): every setting Nomi reads, section by
 * section, each saved here or from the environment. A section saves on its own; storage and Xendit
 * can be tested.
 */
export const load: PageServerLoad = async (event) => ({ configuration: mustLoad(await getJson<Configuration>(event, '/api/admin/configuration')) });

/** A section's name, as the backend names them (lower case, underscores). */
const SECTION = /^[a-z][a-z_]{0,31}$/;

/** The section and its `value_<field>` fields from the form; an empty value goes back to the environment. */
function sectionValues(form: FormData): { section: string; values: Record<string, string> } | null {
	const section = String(form.get('section') ?? '');
	if (!SECTION.test(section)) return null;
	const values: Record<string, string> = {};
	for (const [key, value] of form) if (key.startsWith('value_') && typeof value === 'string') values[key.slice('value_'.length)] = value.trim();
	return { section, values };
}

const failed = (action: string, section: string, problem: { status: number; message: string }) => fail(problem.status >= 400 ? problem.status : 500, { action, section, ok: false as const, problem });

export const actions: Actions = {
	save: async (event) => {
		const read = sectionValues(await event.request.formData());
		if (!read) return refuse('save', m.admin_config_section_invalid(), { section: '' });
		const saved = await send(event, `/api/admin/configuration/${read.section}`, 'PUT', { values: read.values });
		if (!saved.ok) return failed('save', read.section, saved.problem);
		return { action: 'save', ok: true as const, data: { section: read.section } };
	},

	/** Storage tests what's typed (with the saved secret when it's left empty); Xendit tests what's saved. */
	test: async (event) => {
		const read = sectionValues(await event.request.formData());
		if (read?.section !== 'storage' && read?.section !== 'xendit') return refuse('test', m.admin_config_section_invalid(), { section: read?.section ?? '' });
		const tested =
			read.section === 'storage'
				? await send<TestResult>(event, '/api/admin/configuration/storage/test', 'POST', { values: read.values })
				: await send<TestResult>(event, '/api/admin/payment/xendit/test', 'POST');
		if (!tested.ok) return failed('test', read.section, tested.problem);
		return { action: 'test', ok: true as const, data: { section: read.section, ...tested.data } };
	},
};
