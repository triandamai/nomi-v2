// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import ProvidersPanel from './ProvidersPanel.svelte';
import type { ProviderConnection, ProviderDefinition } from '$lib/providers';

// jsdom has no modal dialogs: open/close just toggle the attribute, as the browser would.
beforeAll(() => {
	HTMLDialogElement.prototype.showModal = function (this: HTMLDialogElement) {
		this.setAttribute('open', '');
	};
	HTMLDialogElement.prototype.close = function (this: HTMLDialogElement) {
		this.removeAttribute('open');
		this.dispatchEvent(new Event('close'));
	};
});

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

const providers = [
	provider('anthropic', 'Anthropic', 'popular', { recommended: true, description: 'Claude models' }),
	provider('openai', 'OpenAI', 'popular'),
	provider('gemini', 'Google Gemini', 'popular'),
	provider('groq', 'Groq', 'other'),
	provider('custom', 'Custom OpenAI-compatible provider', 'custom'),
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
const office: ProviderConnection = {
	...openai,
	id: 'c2',
	provider_id: 'custom',
	label: 'Office vLLM',
	auth_type: 'custom_endpoint',
	base_url: 'https://llm.office.test/v1',
	api_key_masked: null,
};

let component: ReturnType<typeof mount> | null = null;

function render(props: Partial<{ connections: ProviderConnection[]; onconnect: () => void; ondisconnect: () => void; onbrowse: () => void }> = {}) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ProvidersPanel, {
		target,
		props: { connections: [openai, office], providers, onconnect: () => {}, ondisconnect: () => {}, onbrowse: () => {}, ...props },
	});
	flushSync();
	return target;
}

const buttonNamed = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === text || b.getAttribute('aria-label') === text);

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

describe('ProvidersPanel', () => {
	it('lists each connection with how it is connected and its key or address', () => {
		const target = render();
		const rows = [...target.querySelectorAll('[data-testid="connection"]')];
		expect(rows).toHaveLength(2);
		expect(rows[0].textContent).toContain('OpenAI');
		expect(rows[0].textContent).toContain('API key');
		expect(rows[0].textContent).toContain('sk-…abcd');
		expect(rows[1].textContent).toContain('Office vLLM');
		expect(rows[1].textContent).toContain('Custom endpoint');
		expect(rows[1].textContent).toContain('https://llm.office.test/v1');
	});

	it('says so when nothing is connected', () => {
		const target = render({ connections: [] });
		expect(target.textContent).toContain('No providers connected yet.');
		expect(target.querySelector('[data-testid="connection"]')).toBeNull();
	});

	it('offers the popular providers not yet connected', () => {
		const onconnect = vi.fn();
		const target = render({ onconnect });
		const popular = target.querySelector('[data-testid="popular"]')!;
		expect(popular.textContent).toContain('Anthropic');
		expect(popular.textContent).toContain('Recommended');
		expect(popular.textContent).toContain('Claude models');
		expect(popular.textContent).toContain('Google Gemini');
		expect(popular.textContent).not.toContain('OpenAI');
		expect(popular.textContent).not.toContain('Groq');
		buttonNamed(popular, 'Connect Anthropic')!.click();
		expect(onconnect).toHaveBeenCalledWith(providers[0]);
	});

	it('opens the full list of providers', () => {
		const onbrowse = vi.fn();
		const target = render({ onbrowse });
		buttonNamed(target, 'Browse all providers')!.click();
		expect(onbrowse).toHaveBeenCalledOnce();
	});

	it('asks before disconnecting', () => {
		const ondisconnect = vi.fn();
		const target = render({ ondisconnect });
		buttonNamed(target, 'Disconnect OpenAI')!.click();
		flushSync();
		const dialog = target.querySelector('dialog')!;
		expect(dialog.hasAttribute('open')).toBe(true);
		expect(dialog.textContent).toContain('Disconnect OpenAI?');
		expect(ondisconnect).not.toHaveBeenCalled();

		buttonNamed(dialog, 'Cancel')!.click();
		flushSync();
		expect(dialog.hasAttribute('open')).toBe(false);
		expect(ondisconnect).not.toHaveBeenCalled();

		buttonNamed(target, 'Disconnect OpenAI')!.click();
		flushSync();
		buttonNamed(dialog, 'Disconnect')!.click();
		flushSync();
		expect(ondisconnect).toHaveBeenCalledWith(openai);
		expect(dialog.hasAttribute('open')).toBe(false);
	});

	it("won't disconnect a provider models still run on", () => {
		const ondisconnect = vi.fn();
		const target = render({ connections: [{ ...openai, models_in_use: ['GPT-5', 'GPT-5 mini'] }], ondisconnect });
		buttonNamed(target, 'Disconnect OpenAI')!.click();
		flushSync();
		const dialog = target.querySelector('dialog')!;
		expect(dialog.textContent).toContain('Remove or switch these models first: GPT-5, GPT-5 mini.');
		const confirm = buttonNamed(dialog, 'Disconnect')!;
		expect(confirm.disabled).toBe(true);
		confirm.click();
		expect(ondisconnect).not.toHaveBeenCalled();
	});
});
