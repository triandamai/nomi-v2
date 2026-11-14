import { m } from '$lib/paraglide/messages';

// A chat model as the admin form writes it, in the two shapes the backend takes
// (llm_models.rs): on one of the instance's connected providers (no key of its own), or with its
// own provider and key the old way. Prices are US dollars per million tokens; media is what the
// model opens, detected from its id unless listed.

export const MEDIA_KINDS = ['image', 'pdf', 'audio', 'video'] as const;

/** The providers a model with its own key can use (llm_models.rs). */
export const OWN_KEY_PROVIDERS = ['anthropic', 'openai', 'openrouter', 'gemini', 'deepseek', 'fake'] as const;

/** The embedding providers the backend knows (settings.rs). */
export const EMBEDDING_PROVIDERS = ['openai', 'gemini', 'cohere', 'fake'] as const;

type Prices = { input_usd_per_mtok: number | null; output_usd_per_mtok: number | null; cached_input_usd_per_mtok: number | null; media_inputs: string[] | null };
export type ModelBody = ({ label: string; connection_id: string; model_id: string } | { label: string; provider: string; model_id: string; api_key: string | null; base_url: string | null }) & Prices;

const text = (form: FormData, name: string): string => {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

/** A price: empty is unset, else dollars, zero or more. NaN when it isn't one. */
function price(form: FormData, name: string): number | null {
	const raw = text(form, name);
	if (!raw) return null;
	const value = Number(raw);
	return Number.isFinite(value) && value >= 0 ? value : Number.NaN;
}

/** The model from the form, or what's wrong with it. */
export function modelFromForm(form: FormData, { editing = false }: { editing?: boolean } = {}): { ok: true; body: ModelBody } | { ok: false; message: string } {
	const label = text(form, 'label');
	const modelId = text(form, 'model_id');
	const own = text(form, 'source') === 'own';
	const connectionId = text(form, 'connection_id');
	const provider = text(form, 'provider');
	if (!label) return { ok: false, message: m.admin_model_label_required() };
	if (own ? !provider : !connectionId) return { ok: false, message: own ? m.admin_model_provider_required() : m.admin_model_connection_required() };
	if (!modelId) return { ok: false, message: m.admin_model_id_required() };
	const prices: Prices = {
		input_usd_per_mtok: price(form, 'input_usd_per_mtok'),
		output_usd_per_mtok: price(form, 'output_usd_per_mtok'),
		cached_input_usd_per_mtok: price(form, 'cached_input_usd_per_mtok'),
		media_inputs: text(form, 'media_mode') === 'listed' ? MEDIA_KINDS.filter((kind) => form.get(`media_${kind}`) === 'on') : null,
	};
	if ([prices.input_usd_per_mtok, prices.output_usd_per_mtok, prices.cached_input_usd_per_mtok].some((value) => Number.isNaN(value))) return { ok: false, message: m.admin_model_price_invalid() };
	if (!own) return { ok: true, body: { label, connection_id: connectionId, model_id: modelId, ...prices } };
	// A new model's key is a string (empty for none); editing, null keeps the saved one.
	const key = text(form, 'api_key');
	return { ok: true, body: { label, provider, model_id: modelId, api_key: key || (editing ? null : ''), base_url: text(form, 'base_url') || null, ...prices } };
}

/** Custom headers, one `Name: value` a line; null when a line isn't one. */
export function headersFromText(raw: string): { name: string; value: string }[] | null {
	const headers: { name: string; value: string }[] = [];
	for (const line of raw
		.split('\n')
		.map((one) => one.trim())
		.filter(Boolean)) {
		const at = line.indexOf(':');
		if (at <= 0) return null;
		const name = line.slice(0, at).trim();
		const value = line.slice(at + 1).trim();
		if (!/^[A-Za-z0-9-]+$/.test(name) || !value) return null;
		headers.push({ name, value });
	}
	return headers;
}
