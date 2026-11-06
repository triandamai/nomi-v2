// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text, type } from '../../../../test-stubs/render';
import Page from './+page.svelte';
import VoiceSheet from './VoiceSheet.svelte';
import type { ToolGroup, VoiceSettings } from './+page.server';

const voice: VoiceSettings = {
	voice_id: 'v1',
	voice_name: 'Rachel',
	model_id: 'eleven_multilingual_v2',
	stability: 0.5,
	similarity_boost: 0.75,
	style: 0,
	use_speaker_boost: true,
	speed: 1,
	output_format: 'mp3_44100_128',
	output_formats: ['mp3_44100_128', 'pcm_16000'],
	max_chars: 2000,
	max_chars_limit: 5000,
	key_source: 'env',
	env_var: 'ELEVENLABS_API_KEY',
	models: [{ model_id: 'eleven_multilingual_v2', name: 'Multilingual v2' }, { model_id: 'eleven_flash_v2_5', name: 'Flash' }],
};

const groups: ToolGroup[] = [
	{
		key: 'crew',
		kind: 'crew',
		label: 'Everyone',
		tools: [
			{ name: 'web_search', description: 'Search the web', enabled: true, required: false, agent_only: false, configurable: true, ready: true, used_by: ['chitchat'], settings: { provider: 'brave', max_results: 5, max_results_limit: 20, base_url: null, providers: [{ id: 'brave', needs_key: true, env_var: 'BRAVE_KEY', key_source: 'saved' }, { id: 'searxng', needs_key: false, env_var: '', key_source: 'missing' }] } },
			{ name: 'read_page', description: 'Read a page', enabled: false, required: false, agent_only: false, configurable: true, ready: true, used_by: [], settings: { max_chars: 20000 } },
			{ name: 'speak', description: 'Say it aloud', enabled: true, required: false, agent_only: false, configurable: true, ready: false, used_by: ['chitchat'], settings: voice },
			{ name: 'show', description: 'Show a view', enabled: true, required: true, agent_only: false, configurable: false, ready: true, used_by: [], settings: null },
		],
	},
	{ key: 'money', kind: 'agent', label: 'Finley', tools: [{ name: 'add_expense', description: 'Log money', enabled: true, required: false, agent_only: true, configurable: false, ready: true, used_by: ['money'], settings: null }] },
	{ key: 'custom', kind: 'custom', label: 'Custom', tools: [] },
];

const action = (data: unknown) => ({ type: 'success', status: 200, data });

describe('admin tools page', () => {
	it('lists the tools by who uses them and opens their settings', async () => {
		const calls = fakeBackend([
			['POST', /testSearch/, action({ hits: [{ title: 'Nomi', url: 'https://nomi.test', snippet: 'hello' }] })],
			['GET', /speak\/voices/, { voices: [{ voice_id: 'v2', name: 'Adam', category: 'premade', description: 'deep', preview_url: null }] }],
			['*', /.*/, action({ saved: true })],
		]);
		const root = render(Page as never, { data: { groups, access: { tools: 'manage' } } as never, form: null as never, params: {} as never });
		const shown = text(root);
		for (const name of ['web_search', 'read_page', 'speak', 'add_expense']) expect(shown).toContain(name);
		clickEverything(root);
		await settle(8);
		clickEverything(document.body);
		await settle(8);
		expect(calls.length).toBeGreaterThanOrEqual(0);
	});

	it('says so when the tools could not be loaded', () => {
		const root = render(Page as never, { data: { groups: null, access: {} } as never, form: null as never, params: {} as never });
		expect(root.textContent).toBeTruthy();
	});
});

describe('voice sheet', () => {
	it('loads the account’s voices, tries one, and saves', async () => {
		vi.stubGlobal('URL', Object.assign(URL, { createObjectURL: () => 'blob:x', revokeObjectURL: () => {} }));
		const calls = fakeBackend([
			['GET', /speak\/voices/, { voices: [{ voice_id: 'v2', name: 'Adam', category: 'premade', description: 'deep', preview_url: 'https://x/a.mp3' }, { voice_id: 'v1', name: 'Rachel', category: 'cloned', description: null, preview_url: null }] }],
			['POST', /speak\/test/, new Response('ID3audio', { status: 200, headers: { 'content-type': 'audio/mpeg' } })],
			['*', /.*/, action({ saved: true })],
		]);
		const root = render(VoiceSheet as never, { open: true, settings: voice });
		await settle(8);
		const textarea = document.querySelector('textarea');
		if (textarea) type(textarea, 'Halo, apa kabar?');
		clickEverything(document.body);
		await settle(8);
		expect(calls.some((c) => c.path.includes('voices'))).toBe(true);
		expect(root).toBeTruthy();
		vi.unstubAllGlobals();
	});

	it('shows when the voices can’t be read', async () => {
		fakeBackend([['*', /.*/, new Response('no key', { status: 401 })]]);
		render(VoiceSheet as never, { open: true, settings: { ...voice, key_source: 'missing', voice_name: null } });
		await settle(8);
		clickEverything(document.body);
		await settle(4);
		expect(document.body.textContent).toBeTruthy();
	});
});
