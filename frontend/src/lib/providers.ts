// The AI providers people (and admins, for everyone) connect with their own keys, and the
// models they pick from them. Shapes match the backend's `/api/llm/providers` and
// `/api/llm/connections` (and the admin `/api/admin/settings/llm/connections`).
import { m } from '$lib/paraglide/messages';
import { searchItems } from '$lib/listView';
import type { MediaInput } from '$lib/types';

export type AuthType = 'api_key' | 'oauth' | 'custom_endpoint';
export type ProviderCategory = 'popular' | 'other' | 'custom';

export interface ProviderDefinition {
	id: string;
	name: string;
	description: string;
	/** `/provider-logos/<id>.svg` */
	icon: string;
	has_logo: boolean;
	category: ProviderCategory;
	recommended: boolean;
	auth_type: AuthType;
	kind: string;
	default_base_url: string | null;
	doc_url: string | null;
	model_count: number;
}

export interface ModelMetadata {
	id: string;
	name: string;
	/** text, image, pdf, audio, video… */
	input: string[];
	/** USD per million tokens. */
	cost: { input: number | null; output: number | null; cache_read: number | null };
	context: number | null;
}

export interface ProviderConnection {
	id: string;
	provider_id: string;
	label: string;
	auth_type: AuthType;
	kind: string;
	base_url: string | null;
	header_names: string[];
	api_key_masked: string | null;
	created_at: string;
	/** Admin connections only: the models that still run on it. */
	models_in_use: string[];
}

export interface ProviderHeader {
	name: string;
	value: string;
}

export interface ConnectInput {
	provider_id: string;
	api_key?: string;
	label?: string;
	base_url?: string;
	headers?: ProviderHeader[];
}

/** A live model from a connection (`POST …/connections/{id}/models`). */
export interface ConnectionModel {
	id: string;
	label: string | null;
}

export interface ProviderGroup {
	category: 'popular' | 'other';
	providers: ProviderDefinition[];
}

export const CUSTOM_PROVIDER_ID = 'custom';

/** Popular, then everything else with the custom provider first; empty groups are left out. */
export function groupProviders(providers: ProviderDefinition[]): ProviderGroup[] {
	const groups: ProviderGroup[] = [
		{ category: 'popular', providers: providers.filter((p) => p.category === 'popular') },
		{
			category: 'other',
			providers: [...providers.filter((p) => p.category === 'custom'), ...providers.filter((p) => p.category === 'other')],
		},
	];
	return groups.filter((g) => g.providers.length > 0);
}

export function searchProviders(providers: ProviderDefinition[], query: string): ProviderDefinition[] {
	return searchItems(providers, query, (p) => `${p.name} ${p.id} ${p.description}`);
}

/** The next highlighted option for an arrow key, wrapping at both ends; -1 when there are none. */
export function stepHighlight(index: number, count: number, key: 'ArrowDown' | 'ArrowUp'): number {
	if (count <= 0) return -1;
	if (index < 0 || index >= count) return key === 'ArrowDown' ? 0 : count - 1;
	return key === 'ArrowDown' ? (index + 1) % count : (index - 1 + count) % count;
}

export function authLabel(authType: AuthType): string {
	if (authType === 'custom_endpoint') return m.prov_auth_custom();
	if (authType === 'oauth') return m.prov_auth_oauth();
	return m.key_api_key();
}

export const logoUrl = (id: string) => `/provider-logos/${id}.svg`;

/** What to call a connection: its provider's name, or the name someone gave a custom endpoint. */
export function connectionName(connection: ProviderConnection, providers: ProviderDefinition[]): string {
	const provider = connection.provider_id === CUSTOM_PROVIDER_ID ? undefined : providers.find((p) => p.id === connection.provider_id);
	return provider?.name ?? (connection.label || connection.provider_id);
}

const ERRORS: Record<string, () => string> = {
	unknown_provider: m.prov_err_unknown_provider,
	key_required: m.prov_err_key_required,
	key_rejected: m.prov_err_key_rejected,
	base_url_required: m.prov_err_base_url_required,
	base_url_invalid: m.prov_err_base_url_invalid,
	header_invalid: m.prov_err_header_invalid,
	label_required: m.prov_err_label_required,
	already_connected: m.prov_err_already_connected,
	unreachable: m.prov_err_unreachable,
	not_found: m.prov_err_not_found,
	in_use: m.prov_err_in_use,
	model_required: m.prov_err_model_required,
};

/** The sentence for an error code from the API; `models` names what still uses a connection. */
export function connectionErrorMessage(code: string | null | undefined, models: string[] = []): string {
	if (code === 'in_use' && models.length > 0) return m.prov_in_use({ models: models.join(', ') });
	return (code && Object.hasOwn(ERRORS, code) ? ERRORS[code] : m.prov_err_generic)();
}

/** The code (and, for `in_use`, the models) from an API error body; `failed` when it has none. */
export function parseApiError(body: string): { error: string; models: string[] } {
	try {
		const parsed = JSON.parse(body) as { error?: unknown; models?: unknown };
		if (typeof parsed?.error === 'string' && parsed.error) {
			const models = Array.isArray(parsed.models) ? parsed.models.filter((v): v is string => typeof v === 'string') : [];
			return { error: parsed.error, models };
		}
	} catch {
		// Not JSON: a proxy or crash page.
	}
	return { error: 'failed', models: [] };
}

/** Form data for the pages' `connect` actions (headers travel as JSON). */
export function connectInputToForm(input: ConnectInput): FormData {
	const form = new FormData();
	form.set('provider_id', input.provider_id);
	if (input.api_key) form.set('api_key', input.api_key);
	if (input.label) form.set('label', input.label);
	if (input.base_url) form.set('base_url', input.base_url);
	if (input.headers?.length) form.set('headers', JSON.stringify(input.headers));
	return form;
}

/** The request body back from that form data, without blank fields; null without a provider. */
export function connectInputFromForm(form: FormData): ConnectInput | null {
	const text = (name: string) => {
		const value = form.get(name);
		return typeof value === 'string' && value.trim() ? value.trim() : undefined;
	};
	const providerId = text('provider_id');
	if (!providerId) return null;
	const input: ConnectInput = { provider_id: providerId };
	const apiKey = text('api_key');
	const label = text('label');
	const baseUrl = text('base_url');
	if (apiKey) input.api_key = apiKey;
	if (label) input.label = label;
	if (baseUrl) input.base_url = baseUrl;
	const headers = readHeaders(text('headers'));
	if (headers.length > 0) input.headers = headers;
	return input;
}

function readHeaders(raw: string | undefined): ProviderHeader[] {
	if (!raw) return [];
	try {
		const parsed = JSON.parse(raw);
		if (!Array.isArray(parsed)) return [];
		return parsed
			.filter((h): h is ProviderHeader => typeof h?.name === 'string' && typeof h?.value === 'string')
			.map((h) => ({ name: h.name.trim(), value: h.value }))
			.filter((h) => h.name);
	} catch {
		return [];
	}
}

const MEDIA: MediaInput[] = ['image', 'pdf', 'audio', 'video'];

/** The catalog's input kinds that Nomi can send a model besides text. */
export const mediaFromInputs = (input: string[]): MediaInput[] => MEDIA.filter((kind) => input.includes(kind));

/** The admin's model form fields from the catalog's entry for the model; blank where it doesn't say. */
export function prefillFromCatalog(meta: ModelMetadata): { inputPrice: string; outputPrice: string; cachedPrice: string; media: MediaInput[] } {
	const price = (value: number | null) => (value === null || value === undefined ? '' : String(value));
	return {
		inputPrice: price(meta.cost.input),
		outputPrice: price(meta.cost.output),
		cachedPrice: price(meta.cost.cache_read),
		media: mediaFromInputs(meta.input),
	};
}

/** "Using {model} from {provider}" for a person's selection of one of their connections' models. */
export function connectionSelectionText(
	selection: { connection_id: string; provider_id: string; label: string; model_id: string },
	connections: ProviderConnection[],
	providers: ProviderDefinition[],
): string {
	const connection = connections.find((c) => c.id === selection.connection_id);
	const provider = connection
		? connectionName(connection, providers)
		: (providers.find((p) => p.id === selection.provider_id)?.name ?? selection.provider_id);
	return m.prov_using({ model: selection.label || selection.model_id, provider });
}
