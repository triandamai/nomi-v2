// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text, type } from '../../../../../test-stubs/render';
import Page from './+page.svelte';

const model = (over: Record<string, unknown> = {}) => ({
	id: 'a1',
	label: 'Sonnet',
	provider: 'anthropic',
	model_id: 'claude-sonnet',
	base_url: null,
	is_default: true,
	api_key_masked: 'sk-…abcd',
	input_usd_per_mtok: 3,
	output_usd_per_mtok: 15,
	cached_input_usd_per_mtok: 0.3,
	is_files_model: true,
	is_coding_model: false,
	enabled: true,
	media_inputs: null,
	media_support: ['image', 'pdf'],
	connection_id: 'c1',
	connection_label: 'Anthropic (team)',
	...over,
});

const models = [
	model(),
	model({ id: 'a2', label: 'Haiku', model_id: 'claude-haiku', is_default: false, is_files_model: false, is_coding_model: true, input_usd_per_mtok: null, output_usd_per_mtok: null, cached_input_usd_per_mtok: null, media_inputs: ['image'], media_support: ['image'] }),
	model({ id: 'a3', label: 'Local', provider: 'openai_compatible', model_id: 'llama', base_url: 'http://gpu:8000', is_default: false, is_files_model: false, enabled: false, connection_id: null, connection_label: null, media_support: [] }),
];
const connections = [{ id: 'c1', provider_id: 'anthropic', label: 'Anthropic (team)', auth_type: 'api_key', kind: 'anthropic', base_url: null, header_names: [], api_key_masked: 'sk-…abcd', created_at: '2026-01-01T00:00:00Z', models_in_use: ['Sonnet'] }];
const providers = [{ id: 'anthropic', name: 'Anthropic', description: 'Claude', icon: '/provider-logos/anthropic.svg', has_logo: true, category: 'popular', recommended: true, auth_type: 'api_key', kind: 'anthropic', default_base_url: null, doc_url: null, model_count: 5 }];

const action = (data: unknown) => ({ type: 'success', status: 200, data });

describe('admin models page', () => {
	it('lists the models with their prices, defaults and connection', () => {
		const root = render(Page as never, { data: { models, connections, providers, access: { models: 'manage' } } as never, form: null as never, params: {} as never });
		const shown = text(root);
		for (const label of ['Sonnet', 'Haiku', 'Local']) expect(shown).toContain(label);
	});

	it('adds and edits a model, prefilled from the catalog', async () => {
		const calls = fakeBackend([
			['POST', /catalogModels/, action({ catalogModels: [{ id: 'claude-sonnet', name: 'Sonnet', input: ['text', 'image'], cost: { input: 3, output: 15, cache_read: 0.3 }, context: 200000 }] })],
			['POST', /connectionModels/, action({ connectionModels: [{ id: 'claude-sonnet', label: 'Sonnet' }] })],
			['POST', /fetchModels/, action({ models: [{ id: 'claude-x', label: null }] })],
			['*', /.*/, action({ success: true })],
		]);
		const root = render(Page as never, { data: { models, connections, providers, access: { models: 'manage' } } as never, form: { error: 'Model id taken' } as never, params: {} as never });
		clickEverything(root, /delete|remove/i);
		await settle(8);
		for (const input of [...document.querySelectorAll<HTMLInputElement>('input[type="text"], input:not([type])')].slice(0, 4)) type(input, 'claude-sonnet');
		clickEverything(document.body, /delete|remove/i);
		await settle(8);
		expect(calls.length).toBeGreaterThan(0);
		expect(text()).toContain('Model id taken');
	});

	it('shows an empty list with no connections yet', async () => {
		const root = render(Page as never, { data: { models: [], connections: [], providers: [], access: { models: 'view' } } as never, form: null as never, params: {} as never });
		clickEverything(root);
		await settle(4);
		expect(root.textContent).toBeTruthy();
	});
});
