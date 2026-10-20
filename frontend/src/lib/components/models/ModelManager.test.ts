// @vitest-environment jsdom
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import type { ModelGroup } from '$lib/modelPicker';
import type { ProviderDefinition } from '$lib/providers';

const postAction = vi.fn();
const reloadData = vi.fn(async () => {});
vi.mock('$lib/providerActions', () => ({ postAction: (...args: unknown[]) => postAction(...args), reloadData: () => reloadData() }));

const { default: ModelManager } = await import('./ModelManager.svelte');

beforeAll(() => {
	HTMLDialogElement.prototype.showModal = function (this: HTMLDialogElement) {
		this.setAttribute('open', '');
	};
	HTMLDialogElement.prototype.close = function (this: HTMLDialogElement) {
		this.removeAttribute('open');
		this.dispatchEvent(new Event('close'));
	};
});

const groups: ModelGroup[] = [
	{
		id: 'conn:c1',
		source: 'connection',
		provider_id: 'openrouter',
		has_logo: true,
		label: 'OpenRouter',
		connection_id: 'c1',
		error: null,
		models: [{ key: 'conn:c1:openai/gpt-5', label: 'GPT-5', model_id: 'openai/gpt-5', free: false, enabled: false }],
	},
];

const catalog: ProviderDefinition[] = [
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

const fetchMock = vi.fn();
let component: ReturnType<typeof mount> | null = null;

function render(onsaved = vi.fn()) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ModelManager, { target, props: { open: true, groups, onsaved } });
	flushSync();
	return target;
}

const settle = async () => {
	for (let i = 0; i < 6; i++) await tick();
	flushSync();
};
const buttonNamed = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === text || b.getAttribute('aria-label') === text);
const dialog = (root: ParentNode, cls: string) => root.querySelector<HTMLDialogElement>(`dialog.${cls}`)!;

beforeEach(() => {
	postAction.mockReset();
	reloadData.mockClear();
	fetchMock.mockReset();
	fetchMock.mockResolvedValue(new Response(JSON.stringify({ providers: catalog }), { status: 200 }));
	vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
	vi.unstubAllGlobals();
});

describe('ModelManager', () => {
	it("saves a switch through the page's setModelVisibility action", async () => {
		postAction.mockResolvedValue({ ok: true, data: {} });
		const onsaved = vi.fn();
		const target = render(onsaved);
		target.querySelector<HTMLButtonElement>('[role="switch"][aria-label="Show GPT-5 in the model selector"]')!.click();
		await settle();
		const [name, body] = postAction.mock.calls[0] as [string, FormData];
		expect(name).toBe('setModelVisibility');
		expect(JSON.parse(String(body.get('changes')))).toEqual([{ key: 'conn:c1:openai/gpt-5', enabled: true }]);
		expect(onsaved).toHaveBeenCalledWith([{ key: 'conn:c1:openai/gpt-5', enabled: true }]);
	});

	it("puts the switch back when the page can't save it", async () => {
		postAction.mockResolvedValue({ ok: false, error: 'failed', models: [] });
		const onsaved = vi.fn();
		const target = render(onsaved);
		const toggle = () => target.querySelector<HTMLButtonElement>('[role="switch"][aria-label="Show GPT-5 in the model selector"]')!;
		toggle().click();
		await settle();
		expect(toggle().getAttribute('aria-checked')).toBe('false');
		expect(onsaved).not.toHaveBeenCalled();
	});

	it('connects a provider in place, then reloads the models', async () => {
		postAction.mockResolvedValue({ ok: true, data: { connection: { id: 'c9' } } });
		const target = render();
		expect(dialog(target, 'provider-dialog').hasAttribute('open')).toBe(false);
		buttonNamed(target, 'Connect provider')!.click();
		await settle();
		expect(fetchMock).toHaveBeenCalledWith('/providers/catalog');
		expect(dialog(target, 'provider-dialog').hasAttribute('open')).toBe(true);

		[...target.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes('OpenAI'))!.click();
		flushSync();
		const key = target.querySelector<HTMLInputElement>('dialog.provider-dialog input[type="password"]')!;
		key.value = 'sk-1';
		key.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		buttonNamed(target, 'Continue')!.click();
		await settle();
		const [name, body] = postAction.mock.calls[0] as [string, FormData];
		expect(name).toBe('connect');
		expect(body.get('provider_id')).toBe('openai');
		expect(reloadData).toHaveBeenCalled();
		expect(dialog(target, 'provider-dialog').hasAttribute('open')).toBe(false);
		// Back on Manage models, with the new provider's models.
		expect(dialog(target, 'models-dialog').hasAttribute('open')).toBe(true);
	});

	it("says why a provider couldn't be connected", async () => {
		postAction.mockResolvedValue({ ok: false, error: 'invalid_key', models: [] });
		const target = render();
		buttonNamed(target, 'Connect provider')!.click();
		await settle();
		[...target.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes('OpenAI'))!.click();
		flushSync();
		const key = target.querySelector<HTMLInputElement>('dialog.provider-dialog input[type="password"]')!;
		key.value = 'nope';
		key.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		buttonNamed(target, 'Continue')!.click();
		await settle();
		expect(reloadData).not.toHaveBeenCalled();
		expect(dialog(target, 'provider-dialog').querySelector('[role="alert"]')).not.toBeNull();
	});
});
