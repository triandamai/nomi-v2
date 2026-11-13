import { describe, expect, it } from 'vitest';
import type { ModelGroup, ProviderDefinition, UsageMonth, UserModels } from '$lib/adapters/api';
import { catalogSections, chatChoices, codingChoices, codingValue, isLocal, pickerCounts, providerCards } from './models';

const groups: ModelGroup[] = [
	{
		id: 'nomi',
		source: 'nomi',
		provider_id: 'nomi',
		has_logo: false,
		label: 'Nomi',
		connection_id: null,
		error: null,
		models: [
			{ key: 'admin:a1', label: 'deepseek-flash', model_id: 'deepseek-flash', free: true, enabled: true },
			{ key: 'admin:a2', label: 'gemini-pro', model_id: 'gemini-pro', free: true, enabled: false },
		],
	},
	{
		id: 'conn:c1',
		source: 'connection',
		provider_id: 'anthropic',
		has_logo: true,
		label: 'Anthropic',
		connection_id: 'c1',
		error: 'unreachable',
		models: [{ key: 'conn:c1:claude-sonnet-5-5', label: 'claude-sonnet-5-5', model_id: 'claude-sonnet-5-5', free: false, enabled: true }],
	},
	{ id: 'conn:c2', source: 'connection', provider_id: 'ollama', has_logo: true, label: 'Ollama', connection_id: 'c2', error: null, models: [] },
];

const usage = {
	by_model: [
		{ admin_model_id: 'a1', label: 'deepseek-flash', provider: 'deepseek', model_id: 'deepseek-flash', source: 'nomi', tokens: 13_500, calls: 9, spend_usd: 0.05 },
		{ admin_model_id: null, label: 'claude-sonnet-5-5', provider: 'anthropic', model_id: 'claude-sonnet-5-5', source: 'own_key', tokens: 2_000, calls: 2, spend_usd: 0 },
	],
} as unknown as UsageMonth;

const connections = [
	{ id: 'c1', provider_id: 'anthropic', label: 'Anthropic', auth_type: 'api_key', kind: 'anthropic', base_url: null, api_key_masked: 'sk-ant-…3f9a', created_at: '' },
	{ id: 'c2', provider_id: 'ollama', label: 'Ollama', auth_type: 'custom_endpoint', kind: 'openai_compatible', base_url: 'http://localhost:11434/v1', api_key_masked: null, created_at: '' },
];

describe('the providers on Models & providers', () => {
	it('are Nomi’s own, then each connection, with what’s in the picker and this month’s tokens', () => {
		const cards = providerCards(groups, usage, 'admin:a1', connections);
		expect(cards.map((card) => [card.label, card.kind, card.shown, card.total, card.error])).toEqual([
			['Nomi', 'included', 1, 2, null],
			['Anthropic', 'key', 1, 1, 'unreachable'],
			['Ollama', 'local', 0, 0, null],
		]);
		expect(cards[0].models).toEqual([
			{ key: 'admin:a1', label: 'deepseek-flash', enabled: true, isDefault: true, month: '13.5K' },
			{ key: 'admin:a2', label: 'gemini-pro', enabled: false, isDefault: false, month: '—' },
		]);
		expect(cards[1]).toMatchObject({ connectionId: 'c1', detail: 'sk-ant-…3f9a', models: [{ month: '2K' }] });
		expect(cards[2]).toMatchObject({ detail: 'http://localhost:11434/v1' });
	});

	it('counts the providers and the models in the picker', () => {
		expect(pickerCounts(providerCards(groups, null, null, connections))).toEqual({ providers: 3, models: 2 });
	});

	it('tells a local server from a hosted one', () => {
		expect([isLocal('http://localhost:11434/v1'), isLocal('http://127.0.0.1:1234'), isLocal('https://api.example.com'), isLocal(null)]).toEqual([true, true, false, false]);
	});
});

describe('the defaults', () => {
	it('offer every model in the picker for new chats', () => {
		expect(chatChoices(groups)).toEqual([
			{ value: 'admin:a1', label: 'deepseek-flash · Nomi' },
			{ value: 'conn:c1:claude-sonnet-5-5', label: 'claude-sonnet-5-5 · Anthropic' },
		]);
	});

	it('offer Nomi’s pick, the same as chats, or one of Nomi’s models for Koda', () => {
		const models: UserModels = {
			admin_models: [{ id: 'a1', label: 'deepseek-flash', provider: 'deepseek', provider_label: 'DeepSeek', model_id: 'deepseek-flash' }],
			coding: { kind: 'admin', admin_model_id: 'a1' },
			default_coding_model_id: 'a1',
		};
		expect(codingChoices(models)).toEqual([
			{ value: 'default', label: 'Nomi’s pick · deepseek-flash' },
			{ value: 'same_as_chat', label: 'Same as new chats' },
			{ value: 'admin:a1', label: 'deepseek-flash · DeepSeek' },
		]);
		expect([codingValue(models.coding), codingValue({ kind: 'default' }), codingValue({ kind: 'same_as_chat' })]).toEqual(['admin:a1', 'default', 'same_as_chat']);
	});
});

describe('the providers to connect', () => {
	const provider = (id: string, name: string, extra: Partial<ProviderDefinition> = {}): ProviderDefinition => ({
		id,
		name,
		description: '',
		has_logo: false,
		category: 'other',
		recommended: false,
		auth_type: 'api_key',
		kind: 'openai_compatible',
		default_base_url: null,
		doc_url: null,
		model_count: 0,
		...extra,
	});
	const catalog = [provider('anthropic', 'Anthropic', { category: 'popular', recommended: true }), provider('ollama', 'Ollama'), provider('abacus', 'Abacus'), provider('302ai', '302.AI'), provider('custom', 'Custom endpoint', { category: 'custom' })];

	it('are the connected ones, the popular ones, then A to Z, the custom endpoint apart', () => {
		expect(catalogSections(catalog, ['ollama'], '').map((section) => [section.label, section.providers.map((one) => one.id)])).toEqual([
			['Connected', ['ollama']],
			['Popular', ['anthropic']],
			['#', ['302ai']],
			['A', ['abacus', 'anthropic']],
			['O', ['ollama']],
		]);
	});

	it('narrow to a search', () => {
		expect(catalogSections(catalog, [], 'ab').map((section) => [section.label, section.providers.map((one) => one.id)])).toEqual([['A', ['abacus']]]);
	});
});
