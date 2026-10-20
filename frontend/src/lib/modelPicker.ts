// The chat's model picker and the Manage models dialog: the person's model options grouped by
// provider (GET /api/llm/model-options), which of them show in the picker, and searching them.
import { searchItems } from '$lib/listView';

/** One model under a provider. `key` is `admin:<uuid>` or `conn:<connection uuid>:<model id>`. */
export interface ModelOption {
	key: string;
	label: string;
	model_id: string;
	free: boolean;
	/** Shown in the chat's model picker. */
	enabled: boolean;
}

export interface ModelGroup {
	/** `nomi:<provider label>` or `conn:<connection uuid>`. */
	id: string;
	source: 'nomi' | 'connection';
	/** Catalog id, for the logo. */
	provider_id: string;
	has_logo: boolean;
	label: string;
	connection_id: string | null;
	/** The provider couldn't be asked for its models just now. */
	error: 'unreachable' | null;
	models: ModelOption[];
}

/** The model a chat uses (or the person's default for new chats). */
export interface ModelChoice {
	key: string;
	label: string;
	model_id: string;
	provider_id: string;
	provider_label: string;
	has_logo: boolean;
	/** Picked for this chat rather than inherited from the person's default. */
	chat_only: boolean;
}

export interface ModelOptions {
	groups: ModelGroup[];
	current: ModelChoice | null;
}

export interface ModelChange {
	key: string;
	enabled: boolean;
}

export type ParsedModelKey = { kind: 'admin'; id: string } | { kind: 'conn'; connectionId: string; modelId: string };

/** Splits a model key; a connection's model id may hold `:` and `/` itself. */
export function parseModelKey(key: string): ParsedModelKey | null {
	if (key.startsWith('admin:')) {
		const id = key.slice('admin:'.length);
		return id ? { kind: 'admin', id } : null;
	}
	if (key.startsWith('conn:')) {
		const rest = key.slice('conn:'.length);
		const colon = rest.indexOf(':');
		if (colon <= 0) return null;
		const modelId = rest.slice(colon + 1);
		return modelId ? { kind: 'conn', connectionId: rest.slice(0, colon), modelId } : null;
	}
	return null;
}

/** What the picker lists: the models switched on, plus the selected one wherever it is. */
export function selectorGroups(groups: ModelGroup[], selectedKey: string | null): ModelGroup[] {
	return groups
		.map((group) => ({ ...group, models: group.models.filter((model) => model.enabled || model.key === selectedKey) }))
		.filter((group) => group.models.length > 0);
}

/** The models holding every word of `query` in their name, ID or provider; groups left empty go. */
export function searchModelGroups(groups: ModelGroup[], query: string): ModelGroup[] {
	if (!query.trim()) return groups;
	return groups
		.map((group) => ({
			...group,
			models: searchItems(group.models, query, (model) => `${group.label} ${group.provider_id} ${model.label} ${model.model_id}`),
		}))
		.filter((group) => group.models.length > 0);
}

/** A provider's master switch: shown on while any of its models is. */
export function groupToggle(group: ModelGroup): 'on' | 'off' | 'mixed' {
	const on = group.models.filter((model) => model.enabled).length;
	if (on === 0) return 'off';
	return on === group.models.length ? 'on' : 'mixed';
}

/** The master switch flips every model of the group, in one request. */
export function groupChanges(group: ModelGroup, enabled: boolean): ModelChange[] {
	return group.models.map((model) => ({ key: model.key, enabled }));
}

/** `groups` with `changes` applied (new objects, so the old ones can be put back). */
export function applyChanges(groups: ModelGroup[], changes: ModelChange[]): ModelGroup[] {
	const next = new Map(changes.map((change) => [change.key, change.enabled]));
	return groups.map((group) =>
		group.models.some((model) => next.has(model.key))
			? { ...group, models: group.models.map((model) => (next.has(model.key) ? { ...model, enabled: next.get(model.key)! } : model)) }
			: group,
	);
}

/** Every model row in display order, for arrow-key navigation (headers aren't stops). */
export function flattenModels(groups: ModelGroup[]): { group: ModelGroup; model: ModelOption }[] {
	return groups.flatMap((group) => group.models.map((model) => ({ group, model })));
}

/** The picked model as the server will describe it, to show before it answers. */
export function choiceFor(groups: ModelGroup[], key: string): ModelChoice | null {
	for (const group of groups) {
		const model = group.models.find((m) => m.key === key);
		if (model) {
			return {
				key,
				label: model.label,
				model_id: model.model_id,
				provider_id: group.provider_id,
				provider_label: group.label,
				has_logo: group.has_logo,
				chat_only: true,
			};
		}
	}
	return null;
}
