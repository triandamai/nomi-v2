import { describe, expect, it } from 'vitest';
import {
	applyChanges,
	choiceFor,
	flattenModels,
	groupChanges,
	groupToggle,
	parseModelKey,
	searchModelGroups,
	selectorGroups,
	type ModelGroup,
} from './modelPicker';

const nomi: ModelGroup = {
	id: 'nomi:Anthropic',
	source: 'nomi',
	provider_id: 'anthropic',
	has_logo: true,
	label: 'Anthropic',
	connection_id: null,
	error: null,
	models: [
		{ key: 'admin:a1', label: 'Claude Sonnet', model_id: 'claude-sonnet', free: false, enabled: true },
		{ key: 'admin:a2', label: 'Claude Haiku', model_id: 'claude-haiku', free: false, enabled: false },
	],
};

const router: ModelGroup = {
	id: 'conn:c1',
	source: 'connection',
	provider_id: 'openrouter',
	has_logo: true,
	label: 'My OpenRouter',
	connection_id: 'c1',
	error: null,
	models: [
		{ key: 'conn:c1:meta-llama/llama-3:free', label: 'Llama 3', model_id: 'meta-llama/llama-3:free', free: true, enabled: true },
		{ key: 'conn:c1:openai/gpt-5', label: 'GPT-5', model_id: 'openai/gpt-5', free: false, enabled: false },
	],
};

const broken: ModelGroup = { ...router, id: 'conn:c2', connection_id: 'c2', label: 'Down', error: 'unreachable', models: [] };

describe('parseModelKey', () => {
	it("reads one of Nomi's models", () => {
		expect(parseModelKey('admin:1234-abcd')).toEqual({ kind: 'admin', id: '1234-abcd' });
	});

	it("splits a connection's model on the first two colons only", () => {
		expect(parseModelKey('conn:c1:meta-llama/llama-3:free')).toEqual({ kind: 'conn', connectionId: 'c1', modelId: 'meta-llama/llama-3:free' });
	});

	it('rejects anything else', () => {
		expect(parseModelKey('')).toBeNull();
		expect(parseModelKey('admin:')).toBeNull();
		expect(parseModelKey('conn:c1')).toBeNull();
		expect(parseModelKey('conn:c1:')).toBeNull();
		expect(parseModelKey('other:x')).toBeNull();
	});
});

describe('selectorGroups', () => {
	it('keeps only the models switched on, dropping groups left empty', () => {
		const off: ModelGroup = { ...nomi, id: 'nomi:Off', models: nomi.models.map((m) => ({ ...m, enabled: false })) };
		const shown = selectorGroups([nomi, router, off, broken], null);
		expect(shown.map((g) => g.id)).toEqual(['nomi:Anthropic', 'conn:c1']);
		expect(shown[0].models.map((m) => m.key)).toEqual(['admin:a1']);
		expect(shown[1].models.map((m) => m.key)).toEqual(['conn:c1:meta-llama/llama-3:free']);
	});

	it('always shows the selected model, even switched off', () => {
		const shown = selectorGroups([nomi, router], 'conn:c1:openai/gpt-5');
		expect(shown[1].models.map((m) => m.key)).toEqual(['conn:c1:meta-llama/llama-3:free', 'conn:c1:openai/gpt-5']);
	});
});

describe('searchModelGroups', () => {
	it('returns everything for an empty query', () => {
		expect(searchModelGroups([nomi, broken], '  ')).toEqual([nomi, broken]);
	});

	it("matches every word against the model's name, its ID and the provider", () => {
		expect(searchModelGroups([nomi, router], 'haiku').map((g) => g.models.map((m) => m.key))).toEqual([['admin:a2']]);
		expect(searchModelGroups([nomi, router], 'openrouter gpt').flatMap((g) => g.models.map((m) => m.key))).toEqual(['conn:c1:openai/gpt-5']);
		expect(searchModelGroups([nomi, router], 'claude gpt')).toEqual([]);
		expect(searchModelGroups([nomi, router], 'my gpt').flatMap((g) => g.models.map((m) => m.key))).toEqual(['conn:c1:openai/gpt-5']);
		expect(searchModelGroups([nomi, router], 'META-LLAMA').flatMap((g) => g.models.map((m) => m.key))).toEqual(['conn:c1:meta-llama/llama-3:free']);
	});

	it("shows a provider's whole group when its name matches", () => {
		expect(searchModelGroups([nomi, router], 'anthropic')).toEqual([nomi]);
	});
});

describe('groupToggle', () => {
	it('is on, off or mixed', () => {
		expect(groupToggle(nomi)).toBe('mixed');
		expect(groupToggle({ ...nomi, models: [nomi.models[0]] })).toBe('on');
		expect(groupToggle({ ...nomi, models: [nomi.models[1]] })).toBe('off');
		expect(groupToggle(broken)).toBe('off');
	});
});

describe('groupChanges', () => {
	it("switches every model of the group, whatever it was", () => {
		expect(groupChanges(nomi, true)).toEqual([
			{ key: 'admin:a1', enabled: true },
			{ key: 'admin:a2', enabled: true },
		]);
	});
});

describe('applyChanges', () => {
	it('updates the changed models and leaves the rest', () => {
		const next = applyChanges([nomi, router], [{ key: 'admin:a2', enabled: true }, { key: 'conn:c1:meta-llama/llama-3:free', enabled: false }]);
		expect(next[0].models.map((m) => m.enabled)).toEqual([true, true]);
		expect(next[1].models.map((m) => m.enabled)).toEqual([false, false]);
		// The originals are untouched (rolling back needs them).
		expect(nomi.models[1].enabled).toBe(false);
	});
});

describe('flattenModels', () => {
	it('lists the rows in display order with their group', () => {
		expect(flattenModels([nomi, router]).map((r) => `${r.group.id}|${r.model.key}`)).toEqual([
			'nomi:Anthropic|admin:a1',
			'nomi:Anthropic|admin:a2',
			'conn:c1|conn:c1:meta-llama/llama-3:free',
			'conn:c1|conn:c1:openai/gpt-5',
		]);
	});
});

describe('choiceFor', () => {
	it('describes a picked model the way the server does', () => {
		expect(choiceFor([nomi, router], 'conn:c1:openai/gpt-5')).toEqual({
			key: 'conn:c1:openai/gpt-5',
			label: 'GPT-5',
			model_id: 'openai/gpt-5',
			provider_id: 'openrouter',
			provider_label: 'My OpenRouter',
			has_logo: true,
			chat_only: true,
		});
		expect(choiceFor([nomi], 'admin:nope')).toBeNull();
	});
});
