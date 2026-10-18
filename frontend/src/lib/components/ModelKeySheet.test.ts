// @vitest-environment jsdom
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import type { ProviderConnection, ProviderDefinition } from '$lib/providers';

const postAction = vi.fn();
const reloadData = vi.fn(async () => {});
vi.mock('$lib/providerActions', () => ({ postAction: (...args: unknown[]) => postAction(...args), reloadData: () => reloadData() }));

const { default: ModelKeySheet } = await import('./ModelKeySheet.svelte');

beforeAll(() => {
	HTMLDialogElement.prototype.showModal = function (this: HTMLDialogElement) {
		this.setAttribute('open', '');
	};
	HTMLDialogElement.prototype.close = function (this: HTMLDialogElement) {
		this.removeAttribute('open');
		this.dispatchEvent(new Event('close'));
	};
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	} as unknown as typeof ResizeObserver;
});

const providers: ProviderDefinition[] = [
	{
		id: 'openai',
		name: 'OpenAI',
		description: 'GPT models',
		icon: '/provider-logos/openai.svg',
		has_logo: true,
		category: 'popular',
		recommended: false,
		auth_type: 'api_key',
		kind: 'openai',
		default_base_url: null,
		doc_url: null,
		model_count: 3,
	},
];

const openai: ProviderConnection = {
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
};

let component: ReturnType<typeof mount> | null = null;

function render(connections: ProviderConnection[]) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ModelKeySheet, { target, props: { open: true, connections, providers } });
	flushSync();
	return target;
}

const buttonNamed = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === text || b.getAttribute('aria-label') === text);
const settle = async () => {
	for (let i = 0; i < 5; i++) await tick();
	flushSync();
};

beforeEach(() => {
	postAction.mockReset();
	reloadData.mockClear();
});

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

describe('ModelKeySheet', () => {
	it('offers to connect a provider when there is none, in place', () => {
		const target = render([]);
		expect(target.textContent).toContain('Connect a provider you have a key for, then pick one of its models.');
		const dialogs = () => [...target.querySelectorAll('dialog')];
		const connectDialog = dialogs().find((d) => d.classList.contains('provider-dialog'))!;
		expect(connectDialog.hasAttribute('open')).toBe(false);
		buttonNamed(target, 'Connect a provider')!.click();
		flushSync();
		expect(connectDialog.hasAttribute('open')).toBe(true);
	});

	it("lists the connection's models and saves the pick as the person's model", async () => {
		postAction.mockImplementation(async (name: string) =>
			name === 'connectionModels'
				? { ok: true, data: { connectionModels: [{ id: 'gpt-5', label: 'GPT-5' }, { id: 'gpt-5-mini', label: null }] } }
				: { ok: true, data: { modelSelected: true } },
		);
		const target = render([openai]);
		await settle();
		const [name, body] = postAction.mock.calls[0] as [string, FormData];
		expect(name).toBe('connectionModels');
		expect(body.get('connection_id')).toBe('c1');
		expect(target.querySelector<HTMLInputElement>('input[name="model_id"]')?.value).toBe('gpt-5');

		buttonNamed(target, 'Use this model')!.click();
		await settle();
		const [saveName, saveBody] = postAction.mock.calls[1] as [string, FormData];
		expect(saveName).toBe('selectConnectionModel');
		expect(saveBody.get('connection_id')).toBe('c1');
		expect(saveBody.get('model_id')).toBe('gpt-5');
		expect(saveBody.get('label')).toBe('GPT-5');
		expect(reloadData).toHaveBeenCalled();
	});

	it('falls back to typing the model ID when the list fails', async () => {
		postAction.mockResolvedValue({ ok: false, error: 'unreachable', models: [] });
		const target = render([openai]);
		await settle();
		expect(target.textContent).toContain("Couldn't reach the provider.");
		expect(target.querySelector<HTMLInputElement>('input[name="model_id"]')?.type).toBe('text');
	});
});
