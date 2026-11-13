import { m } from '$lib/paraglide/messages';
import type { SearchTest, ToolGroup, VoiceOptions } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Tools (tools: view; write changes): what the crew can use, by who uses it, each switched on or
 * off, and the settings of the configurable ones: web search, reading a page, and the voice. The
 * voices come from the voice service with the saved key; without it, the voice is typed by id.
 */
export const load: PageServerLoad = async (event) => {
	const tools = mustLoad(await getJson<{ groups: ToolGroup[] }>(event, '/api/admin/tools'));
	const groups = tools.ok ? tools.data.groups : [];
	const speaks = groups.some((group) => group.tools.some((tool) => tool.name === 'speak'));
	const voices = speaks ? await getJson<VoiceOptions>(event, '/api/admin/tools/speak/voices') : null;
	return { tools: tools.ok ? { ok: true as const, data: groups } : tools, voices: voices?.ok ? voices.data : null };
};

/** A number from the form; NaN when it isn't one, null when it's empty. */
function number(form: FormData, name: string): number | null {
	const raw = field(form, name).replace(/[\s,_]/g, '');
	if (!raw) return null;
	const value = Number(raw);
	return Number.isFinite(value) ? value : Number.NaN;
}

export const actions: Actions = {
	toggle: async (event) => {
		const form = await event.request.formData();
		const name = field(form, 'name');
		if (!name) return refuse('toggle', m.admin_tool_required());
		return act(event, 'toggle', `/api/admin/tools/${encodeURIComponent(name)}`, 'PUT', { enabled: form.get('enabled') === 'on' });
	},

	saveSearch: async (event) => {
		const form = await event.request.formData();
		const results = number(form, 'max_results');
		if (results !== null && (Number.isNaN(results) || !Number.isInteger(results) || results < 1)) return refuse('saveSearch', m.admin_tool_number_invalid());
		return act(event, 'saveSearch', '/api/admin/tools/web_search/settings', 'PUT', {
			provider: field(form, 'provider') || 'tavily',
			max_results: results ?? 5,
			base_url: field(form, 'base_url') || null,
			api_key: field(form, 'api_key') || null,
			clear_api_key: form.get('clear_api_key') === 'on',
		});
	},

	saveReadPage: async (event) => {
		const chars = number(await event.request.formData(), 'max_chars');
		if (chars === null || Number.isNaN(chars) || !Number.isInteger(chars)) return refuse('saveReadPage', m.admin_tool_number_invalid());
		return act(event, 'saveReadPage', '/api/admin/tools/read_web_page/settings', 'PUT', { max_chars: chars });
	},

	saveVoice: async (event) => {
		const form = await event.request.formData();
		const tuning = { stability: number(form, 'stability'), similarity_boost: number(form, 'similarity_boost'), style: number(form, 'style'), speed: number(form, 'speed'), max_chars: number(form, 'max_chars') };
		if (Object.values(tuning).some((value) => value !== null && Number.isNaN(value))) return refuse('saveVoice', m.admin_tool_number_invalid());
		return act(event, 'saveVoice', '/api/admin/tools/speak/settings', 'PUT', {
			voice_id: field(form, 'voice_id'),
			voice_name: field(form, 'voice_name') || null,
			model_id: field(form, 'model_id'),
			stability: tuning.stability ?? 0.5,
			similarity_boost: tuning.similarity_boost ?? 0.75,
			style: tuning.style ?? 0,
			use_speaker_boost: form.get('use_speaker_boost') === 'on',
			speed: tuning.speed ?? 1,
			output_format: field(form, 'output_format') || 'mp3_44100_128',
			max_chars: Math.round(tuning.max_chars ?? 2500),
			api_key: field(form, 'api_key') || null,
			clear_api_key: form.get('clear_api_key') === 'on',
		});
	},

	testSearch: async (event) => {
		const query = field(await event.request.formData(), 'query');
		return act<SearchTest>(event, 'testSearch', '/api/admin/tools/web_search/test', 'POST', { query: query || null });
	},
};
