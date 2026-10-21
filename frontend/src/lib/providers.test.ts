import { describe, expect, it } from 'vitest';
import {
	authLabel,
	connectInputFromForm,
	connectInputToForm,
	connectionErrorMessage,
	connectionName,
	groupProviders,
	logoUrl,
	mediaFromInputs,
	prefillFromCatalog,
	parseApiError,
	searchProviders,
	stepHighlight,
	type ProviderConnection,
	type ProviderDefinition,
} from './providers';

function provider(id: string, name: string, category: ProviderDefinition['category'], extra: Partial<ProviderDefinition> = {}): ProviderDefinition {
	return {
		id,
		name,
		description: `${name} models`,
		icon: `/provider-logos/${id}.svg`,
		has_logo: true,
		category,
		recommended: false,
		auth_type: category === 'custom' ? 'custom_endpoint' : 'api_key',
		kind: 'openai',
		default_base_url: null,
		doc_url: null,
		model_count: 3,
		...extra,
	};
}

const catalog = [
	provider('anthropic', 'Anthropic', 'popular'),
	provider('openai', 'OpenAI', 'popular'),
	provider('deepseek', 'DeepSeek', 'other', { description: 'Reasoning models from China' }),
	provider('groq', 'Groq', 'other'),
	provider('custom', 'Custom OpenAI-compatible provider', 'custom'),
];

function connection(extra: Partial<ProviderConnection> = {}): ProviderConnection {
	return {
		id: 'c1',
		provider_id: 'openai',
		label: 'OpenAI',
		auth_type: 'api_key',
		kind: 'openai',
		base_url: null,
		header_names: [],
		api_key_masked: 'sk-…abcd',
		created_at: '2026-10-01T00:00:00Z',
		models_in_use: [],
		...extra,
	};
}

describe('groupProviders', () => {
	it('puts popular first, then the rest with the custom provider leading Other', () => {
		const groups = groupProviders(catalog);
		expect(groups.map((g) => g.category)).toEqual(['popular', 'other']);
		expect(groups[0].providers.map((p) => p.id)).toEqual(['anthropic', 'openai']);
		expect(groups[1].providers.map((p) => p.id)).toEqual(['custom', 'deepseek', 'groq']);
	});

	it('drops groups with nothing in them', () => {
		expect(groupProviders(catalog.filter((p) => p.category !== 'popular')).map((g) => g.category)).toEqual(['other']);
		expect(groupProviders([])).toEqual([]);
	});
});

describe('searchProviders', () => {
	it('matches every word against the name, id and description', () => {
		expect(searchProviders(catalog, 'open').map((p) => p.id)).toEqual(['openai', 'custom']);
		expect(searchProviders(catalog, 'reasoning china').map((p) => p.id)).toEqual(['deepseek']);
		expect(searchProviders(catalog, 'GROQ').map((p) => p.id)).toEqual(['groq']);
	});

	it('returns everything for an empty query', () => {
		expect(searchProviders(catalog, '  ')).toHaveLength(catalog.length);
	});
});

describe('stepHighlight', () => {
	it('moves down and wraps to the top', () => {
		expect(stepHighlight(0, 3, 'ArrowDown')).toBe(1);
		expect(stepHighlight(2, 3, 'ArrowDown')).toBe(0);
	});

	it('moves up and wraps to the bottom', () => {
		expect(stepHighlight(1, 3, 'ArrowUp')).toBe(0);
		expect(stepHighlight(0, 3, 'ArrowUp')).toBe(2);
	});

	it('starts from the ends when nothing is highlighted, and stays at -1 with no options', () => {
		expect(stepHighlight(-1, 3, 'ArrowDown')).toBe(0);
		expect(stepHighlight(-1, 3, 'ArrowUp')).toBe(2);
		expect(stepHighlight(0, 0, 'ArrowDown')).toBe(-1);
		expect(stepHighlight(-1, 0, 'ArrowUp')).toBe(-1);
	});
});

describe('labels', () => {
	it('names each way of connecting', () => {
		expect(authLabel('api_key')).toBe('API key');
		expect(authLabel('custom_endpoint')).toBe('Custom endpoint');
		expect(authLabel('oauth')).toBe('Signed in');
	});

	it('points at the monochrome logo', () => {
		expect(logoUrl('openai')).toBe('/provider-logos/openai.svg');
	});

	it('names a connection by its provider, or by its own label for a custom one', () => {
		expect(connectionName(connection(), catalog)).toBe('OpenAI');
		expect(connectionName(connection({ provider_id: 'custom', label: 'Office vLLM' }), catalog)).toBe('Office vLLM');
		expect(connectionName(connection({ provider_id: 'gone', label: 'Old one' }), catalog)).toBe('Old one');
		expect(connectionName(connection({ provider_id: 'gone', label: '' }), catalog)).toBe('gone');
	});
});

describe('connectionErrorMessage', () => {
	it('explains each error the API sends', () => {
		expect(connectionErrorMessage('key_rejected')).toBe("The provider didn't accept that key. Check it and try again.");
		expect(connectionErrorMessage('key_required')).toBe('Enter an API key.');
		expect(connectionErrorMessage('already_connected')).toBe('That provider is already connected.');
		expect(connectionErrorMessage('model_required')).toBe('Pick a model.');
		expect(connectionErrorMessage('unreachable')).toBe("Couldn't reach the provider. Check the address or try again later.");
		expect(connectionErrorMessage('base_url_invalid')).toContain('base URL');
		for (const code of ['unknown_provider', 'base_url_required', 'header_invalid', 'label_required', 'not_found']) {
			expect(connectionErrorMessage(code)).not.toBe(connectionErrorMessage('something_else'));
		}
	});

	it('lists the models still using a connection', () => {
		expect(connectionErrorMessage('in_use', ['Claude', 'GPT-5'])).toBe('Remove or switch these models first: Claude, GPT-5.');
	});

	it('falls back to a general message', () => {
		expect(connectionErrorMessage('teapot')).toBe('Something went wrong. Try again.');
		expect(connectionErrorMessage(null)).toBe('Something went wrong. Try again.');
	});
});

describe('parseApiError', () => {
	it('reads the code and the models from a JSON error body', () => {
		expect(parseApiError('{"error":"in_use","models":["Claude"]}')).toEqual({ error: 'in_use', models: ['Claude'] });
		expect(parseApiError('{"error":"key_rejected","detail":"401 from upstream"}')).toEqual({ error: 'key_rejected', models: [] });
	});

	it('gives a generic code for anything else', () => {
		expect(parseApiError('Internal Server Error')).toEqual({ error: 'failed', models: [] });
		expect(parseApiError('{"message":"nope"}')).toEqual({ error: 'failed', models: [] });
		expect(parseApiError('')).toEqual({ error: 'failed', models: [] });
	});
});

describe('connect form', () => {
	it('round-trips a key connection through form data', () => {
		const input = { provider_id: 'openai', api_key: 'sk-1' };
		expect(connectInputFromForm(connectInputToForm(input))).toEqual(input);
	});

	it('round-trips a custom endpoint with its headers', () => {
		const input = {
			provider_id: 'custom',
			label: 'Office vLLM',
			base_url: 'https://llm.example.com/v1',
			api_key: 'k',
			headers: [{ name: 'X-Org', value: '42' }],
		};
		expect(connectInputFromForm(connectInputToForm(input))).toEqual(input);
	});

	it('drops blank fields and refuses a form without a provider', () => {
		const form = new FormData();
		form.set('provider_id', 'custom');
		form.set('label', '  ');
		form.set('base_url', 'https://x.test');
		form.set('headers', 'not json');
		expect(connectInputFromForm(form)).toEqual({ provider_id: 'custom', base_url: 'https://x.test' });
		expect(connectInputFromForm(new FormData())).toBeNull();
	});
});

describe('mediaFromInputs', () => {
	it('keeps the kinds of file Nomi can send besides text', () => {
		expect(mediaFromInputs(['text', 'image', 'pdf', 'audio', 'video'])).toEqual(['image', 'pdf', 'audio', 'video']);
		expect(mediaFromInputs(['text'])).toEqual([]);
		expect(mediaFromInputs(['image', 'svg'])).toEqual(['image']);
	});
});

describe('prefillFromCatalog', () => {
	it("fills the price fields (USD per 1M tokens) and what the model opens", () => {
		const meta = { id: 'gpt-5', name: 'GPT-5', input: ['text', 'image', 'pdf'], cost: { input: 1.25, output: 10, cache_read: 0.125 }, context: 400000 };
		expect(prefillFromCatalog(meta)).toEqual({ inputPrice: '1.25', outputPrice: '10', cachedPrice: '0.125', media: ['image', 'pdf'] });
	});

	it('leaves unknown prices blank', () => {
		const meta = { id: 'x', name: 'X', input: ['text'], cost: { input: null, output: 0, cache_read: null }, context: null };
		expect(prefillFromCatalog(meta)).toEqual({ inputPrice: '', outputPrice: '0', cachedPrice: '', media: [] });
	});
});
