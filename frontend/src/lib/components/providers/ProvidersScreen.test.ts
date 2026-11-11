// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { render, text } from '../../../test-stubs/render';
import ProvidersScreen from './ProvidersScreen.svelte';
import type { ProviderConnection, ProviderDefinition } from '$lib/providers';

const providers = [
	{
		id: 'anthropic',
		name: 'Anthropic',
		description: 'Claude models',
		kind: 'llm',
		icon: 'anthropic',
		auth_type: 'api_key',
		fields: [{ name: 'api_key', label: 'API Key', type: 'password', required: true }],
		models: [{ id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet' }],
	},
] as unknown as ProviderDefinition[];

const connections = [
	{
		id: 'conn-1',
		provider_id: 'anthropic',
		name: 'My Anthropic',
		status: 'connected',
		created_at: '2026-01-01',
		updated_at: '2026-01-01',
	},
] as unknown as ProviderConnection[];

describe('ProvidersScreen', () => {
	it('renders providers and connections list', () => {
		const root = render(ProvidersScreen, {
			providers,
			connections,
			loadFailed: false,
		});
		expect(text(root)).toContain('Anthropic');
	});

	it('renders loadFailed error message', () => {
		const root = render(ProvidersScreen, {
			providers: [],
			connections: [],
			loadFailed: true,
		});
		expect(root.querySelector('[role="alert"]')).toBeTruthy();
	});
});
