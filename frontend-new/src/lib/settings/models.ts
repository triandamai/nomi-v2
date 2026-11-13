import { m } from '$lib/paraglide/messages';
import type { ModelGroup, OwnConnection, ProviderDefinition, UsageMonth, UserModels } from '$lib/adapters/api';
import { formatTokens } from '$lib/adapters/shell';

// Settings › Models & providers (App / Settings / Models & providers RbTgo, Connect voUxB): Nomi's
// own models and each connected provider with which of their models are in the chat's picker and
// this month's tokens; the defaults for new chats and for Koda; the providers to connect, as the
// Connect dialog lists them. Model keys are the backend's: "admin:{id}" for Nomi's,
// "conn:{connection}:{model}" for a connection's.

export type ProviderKind = 'included' | 'key' | 'local';
export type ProviderModel = { key: string; label: string; enabled: boolean; isDefault: boolean; month: string };
export type ProviderCard = {
	id: string;
	label: string;
	kind: ProviderKind;
	providerId: string;
	connectionId: string | null;
	/** The masked key, or the address of a local server. */
	detail: string | null;
	error: 'unreachable' | null;
	shown: number;
	total: number;
	models: ProviderModel[];
};

/** A server on this machine (Ollama, LM Studio), which costs nothing and works offline. */
export function isLocal(url: string | null): boolean {
	return !!url && /^https?:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1\])(:|\/|$)/.test(url);
}

export function providerCards(groups: ModelGroup[], usage: UsageMonth | null, currentKey: string | null, connections: OwnConnection[]): ProviderCard[] {
	const tokens = (key: string, modelId: string) => {
		const used = usage?.by_model.find((model) =>
			key.startsWith('admin:') ? model.admin_model_id === key.slice(6) : model.source === 'own_key' && model.model_id === modelId,
		);
		return used ? formatTokens(used.tokens) : '—';
	};
	return groups.map((group) => {
		const connection = connections.find((one) => one.id === group.connection_id);
		const kind: ProviderKind = group.source === 'nomi' ? 'included' : isLocal(connection?.base_url ?? null) ? 'local' : 'key';
		const models = group.models.map((model) => ({ key: model.key, label: model.label, enabled: model.enabled, isDefault: model.key === currentKey, month: tokens(model.key, model.model_id) }));
		return {
			id: group.id,
			label: group.label,
			kind,
			providerId: group.provider_id,
			connectionId: group.connection_id,
			detail: kind === 'local' ? (connection?.base_url ?? null) : (connection?.api_key_masked ?? null),
			error: group.error,
			shown: models.filter((model) => model.enabled).length,
			total: models.length,
			models,
		};
	});
}

/** The top bar's "3 providers · 7 models in picker". */
export function pickerCounts(cards: ProviderCard[]): { providers: number; models: number } {
	return { providers: cards.length, models: cards.reduce((sum, card) => sum + card.shown, 0) };
}

/** New chats can start with any model in the picker. */
export function chatChoices(groups: ModelGroup[]): { value: string; label: string }[] {
	return groups.flatMap((group) => group.models.filter((model) => model.enabled).map((model) => ({ value: model.key, label: `${model.label} · ${group.label}` })));
}

/** Koda builds with Nomi's pick (an admin's choice), the chat's model, or one of Nomi's models. */
export function codingChoices(models: UserModels): { value: string; label: string }[] {
	const pick = models.admin_models.find((model) => model.id === models.default_coding_model_id);
	return [
		{ value: 'default', label: pick ? m.models_coding_default_named({ model: pick.label }) : m.models_coding_default() },
		{ value: 'same_as_chat', label: m.models_coding_same() },
		...models.admin_models.map((model) => ({ value: `admin:${model.id}`, label: `${model.label} · ${model.provider_label}` })),
	];
}

export function codingValue(coding: UserModels['coding']): string {
	return coding.kind === 'admin' ? `admin:${coding.admin_model_id}` : coding.kind;
}

/** The Connect dialog's list: what's connected, the popular ones, then A to Z ("#" for digits). */
export function catalogSections(catalog: ProviderDefinition[], connected: string[], query: string): { label: string; providers: ProviderDefinition[] }[] {
	const wanted = query.trim().toLowerCase();
	const listed = catalog.filter((provider) => provider.category !== 'custom' && (!wanted || provider.name.toLowerCase().includes(wanted) || provider.id.includes(wanted)));
	const byName = (a: ProviderDefinition, b: ProviderDefinition) => a.name.localeCompare(b.name);
	const letters = new Map<string, ProviderDefinition[]>();
	for (const provider of [...listed].sort(byName)) {
		const first = provider.name.charAt(0).toUpperCase();
		const letter = /[A-Z]/.test(first) ? first : '#';
		letters.set(letter, [...(letters.get(letter) ?? []), provider]);
	}
	const sections = [
		...(!wanted ? [{ label: m.models_section_connected(), providers: listed.filter((provider) => connected.includes(provider.id)) }] : []),
		...(!wanted ? [{ label: m.models_section_popular(), providers: listed.filter((provider) => provider.category === 'popular') }] : []),
		...[...letters].sort(([a], [b]) => (a === '#' ? -1 : b === '#' ? 1 : a.localeCompare(b))).map(([label, providers]) => ({ label, providers })),
	];
	return sections.filter((section) => section.providers.length > 0);
}
