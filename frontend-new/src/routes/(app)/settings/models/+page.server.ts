import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { UserModels } from '$lib/adapters/api';
import { connectionModels, connectProvider, disconnectProvider, loadModels, setCodingModel, setDefaultModel, setShown, type Sent } from '$lib/server/models';
import { pickerCounts, providerCards } from '$lib/settings/models';
import type { Actions, PageServerLoad } from './$types';

/** Settings › Models & providers (App / Settings / Models & providers RbTgo, Connect voUxB). */
export const load: PageServerLoad = async (event) => {
	event.depends('app:models');
	const data = await loadModels(event);
	if (!data) error(503, { message: m.models_load_failed() });
	const counts = pickerCounts(providerCards(data.options.groups, data.usage, data.options.current?.key ?? null, data.connections));
	const meta = [counts.providers === 1 ? m.models_meta_provider() : m.models_meta_providers({ count: counts.providers }), counts.models === 1 ? m.models_meta_model() : m.models_meta_models({ count: counts.models })].join(' · ');
	// The top bar's Connect a provider opens the dialog (`?connect`).
	return { title: m.settings_models(), meta, connect: '?connect', ...data };
};

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();
const refused = (sent: Extract<Sent<unknown>, { ok: false }>) => fail(sent.status, { message: sent.message });

function parse<T>(raw: string, valid: (value: unknown) => boolean): T | null {
	try {
		const value = JSON.parse(raw) as unknown;
		return valid(value) ? (value as T) : null;
	} catch {
		return null;
	}
}

/** The provider's own reason, in plain words: a refused key, an address that can't be reached. */
function connectFailure(sent: Extract<Sent<unknown>, { ok: false }>) {
	const detail = sent.message;
	const message = sent.code === 'key_rejected' ? m.models_key_rejected({ detail }) : sent.code === 'unreachable' ? m.models_unreachable({ detail }) : detail || m.action_failed();
	return fail(sent.status, { field: 'key', message });
}

export const actions: Actions = {
	setDefault: async (event) => {
		const key = field(await event.request.formData(), 'key');
		if (!/^(admin|conn):/.test(key)) return fail(400, { message: m.action_failed() });
		const sent = await setDefaultModel(event, key);
		return sent.ok ? { saved: 'default' } : refused(sent);
	},
	setCoding: async (event) => {
		const value = field(await event.request.formData(), 'value');
		const selection: UserModels['coding'] | null =
			value === 'default' || value === 'same_as_chat' ? { kind: value } : value.startsWith('admin:') ? { kind: 'admin', admin_model_id: value.slice(6) } : null;
		if (!selection) return fail(400, { message: m.action_failed() });
		const sent = await setCodingModel(event, selection);
		return sent.ok ? { saved: 'coding' } : refused(sent);
	},
	setShown: async (event) => {
		const changes = parse<{ key: string; enabled: boolean }[]>(field(await event.request.formData(), 'changes'), (value) =>
			Array.isArray(value) && value.every((change) => typeof change?.key === 'string' && typeof change?.enabled === 'boolean'),
		);
		if (!changes) return fail(400, { message: m.action_failed() });
		const sent = await setShown(event, changes);
		return sent.ok ? { saved: 'shown' } : refused(sent);
	},
	/** The backend tries the key (or the local address) with the provider before keeping it. */
	connect: async (event) => {
		const form = await event.request.formData();
		const [provider_id, api_key, base_url] = [field(form, 'provider_id'), field(form, 'api_key'), field(form, 'base_url')];
		if (!provider_id) return fail(400, { message: m.action_failed() });
		const sent = await connectProvider(event, { provider_id, ...(api_key && { api_key }), ...(base_url && { base_url }) });
		if (!sent.ok) return connectFailure(sent);
		const offered = await connectionModels(event, sent.data.id);
		return { connection: { id: sent.data.id, label: sent.data.label }, models: offered.ok ? offered.data.models : [] };
	},
	showModels: async (event) => {
		const form = await event.request.formData();
		const id = field(form, 'connection_id');
		const ids = parse<string[]>(field(form, 'models'), (value) => Array.isArray(value) && value.every((one) => typeof one === 'string'));
		if (!id || !ids) return fail(400, { message: m.action_failed() });
		const sent = await setShown(event, ids.map((model) => ({ key: `conn:${id}:${model}`, enabled: true })));
		return sent.ok ? { saved: 'shown' } : refused(sent);
	},
	disconnect: async (event) => {
		const id = field(await event.request.formData(), 'id');
		const sent = await disconnectProvider(event, id);
		return sent.ok ? { disconnected: id } : refused(sent);
	},
};
