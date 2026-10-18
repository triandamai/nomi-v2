// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import ConnectProviderDialog from './ConnectProviderDialog.svelte';
import type { ConnectInput, ProviderDefinition } from '$lib/providers';

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
	provider('anthropic', 'Anthropic', 'popular', { doc_url: 'https://docs.anthropic.test/keys' }),
	provider('openai', 'OpenAI', 'popular'),
	provider('gemini', 'Google Gemini', 'popular'),
	provider('deepseek', 'DeepSeek', 'other'),
	provider('groq', 'Groq', 'other', { description: 'Fast inference' }),
	provider('custom', 'Custom OpenAI-compatible provider', 'custom', { description: 'Any OpenAI-compatible API' }),
];

let component: ReturnType<typeof mount> | null = null;

type Props = { connectedIds: string[]; initialProviderId: string | null; error: string | null; busy: boolean; onsubmit: (input: ConnectInput) => void };

function render(props: Partial<Props> = {}) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ConnectProviderDialog, {
		target,
		props: { open: true, providers, connectedIds: [], onsubmit: () => {}, ...props },
	});
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const search = (root: ParentNode) => root.querySelector<HTMLInputElement>('input[role="combobox"]')!;
const optionNames = (root: ParentNode) =>
	[...root.querySelectorAll('[role="option"]')].map((o) => o.querySelector('[data-name]')?.textContent?.trim());
const buttonNamed = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === text || b.getAttribute('aria-label') === text);
const fieldLabelled = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('label')].find((l) => l.querySelector('span')?.textContent?.trim() === text)?.querySelector('input') as HTMLInputElement | undefined;

function type(input: HTMLInputElement, value: string) {
	input.value = value;
	input.dispatchEvent(new Event('input', { bubbles: true }));
	flushSync();
}

function press(input: HTMLElement, key: string) {
	input.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
	flushSync();
}

function pick(root: ParentNode, name: string) {
	const option = [...root.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.querySelector('[data-name]')?.textContent?.trim() === name)!;
	option.click();
	flushSync();
}

describe('ConnectProviderDialog', () => {
	it('lists the providers under Popular, then Other with the custom provider first', () => {
		const target = render();
		const groups = [...target.querySelectorAll('[role="group"]')];
		expect(groups.map((g) => g.getAttribute('aria-label'))).toEqual(['Popular', 'Other']);
		expect(optionNames(groups[0])).toEqual(['Anthropic', 'OpenAI', 'Google Gemini']);
		expect(optionNames(groups[1])).toEqual(['Custom OpenAI-compatible provider', 'DeepSeek', 'Groq']);
		expect(groups[1].textContent).toContain('Custom');
		expect(groups[1].textContent).toContain('Fast inference');
	});

	it('filters as you type', () => {
		const target = render();
		type(search(target), 'groq');
		expect(optionNames(target)).toEqual(['Groq']);
		expect(target.querySelectorAll('[role="group"]')).toHaveLength(1);
		type(search(target), 'zzz');
		expect(optionNames(target)).toEqual([]);
		expect(target.textContent).toContain('Nothing matches “zzz”.');
	});

	it("says so when the providers couldn't be loaded", () => {
		const target = document.createElement('div');
		document.body.append(target);
		component = mount(ConnectProviderDialog, { target, props: { open: true, providers: [], connectedIds: [], onsubmit: () => {} } });
		flushSync();
		expect(target.textContent).toContain("Couldn't load providers.");
		expect(target.textContent).not.toContain('Nothing matches');
	});

	it("marks connected providers and doesn't open them", () => {
		const target = render({ connectedIds: ['openai'] });
		const openai = [...target.querySelectorAll('[role="option"]')].find((o) => o.textContent?.includes('OpenAI') && !o.textContent?.includes('Custom'))!;
		expect(openai.getAttribute('aria-disabled')).toBe('true');
		expect(openai.textContent).toContain('Connected');
		(openai as HTMLElement).click();
		flushSync();
		expect(target.querySelector('h2')?.textContent).toBe('Connect provider');
	});

	it('moves the highlight with the arrow keys, wrapping, and opens it with Enter', () => {
		const target = render({ connectedIds: ['openai'] });
		const input = search(target);
		const active = () => target.querySelector(`#${input.getAttribute('aria-activedescendant')}`)?.querySelector('[data-name]')?.textContent?.trim();
		expect(active()).toBe('Anthropic');
		press(input, 'ArrowDown');
		// OpenAI is connected, so the highlight skips it.
		expect(active()).toBe('Google Gemini');
		press(input, 'ArrowUp');
		press(input, 'ArrowUp');
		expect(active()).toBe('Groq');
		press(input, 'ArrowDown');
		press(input, 'ArrowDown');
		expect(active()).toBe('Google Gemini');
		press(input, 'Enter');
		expect(target.querySelector('h2')?.textContent).toBe('Connect Google Gemini');
	});

	it('closes on Escape and with the close button', () => {
		let target = render();
		const dialog = target.querySelector('dialog')!;
		expect(dialog.hasAttribute('open')).toBe(true);
		press(search(target), 'Escape');
		expect(dialog.hasAttribute('open')).toBe(false);

		unmount(component!);
		document.body.innerHTML = '';
		target = render();
		buttonNamed(target, 'Close')!.click();
		flushSync();
		expect(target.querySelector('dialog')!.hasAttribute('open')).toBe(false);
	});

	it('asks for the key, linking to the docs, and submits it', () => {
		const onsubmit = vi.fn();
		const target = render({ onsubmit });
		pick(target, 'Anthropic');
		expect(target.querySelector('h2')?.textContent).toBe('Connect Anthropic');
		expect(target.textContent).toContain('Enter your Anthropic API key to connect your account and use Anthropic models in Nomi.');
		expect(target.querySelector<HTMLAnchorElement>('a[href="https://docs.anthropic.test/keys"]')).not.toBeNull();

		const key = fieldLabelled(target, 'Anthropic API key')!;
		expect(key.type).toBe('password');
		const cont = buttonNamed(target, 'Continue')!;
		expect(cont.disabled).toBe(true);
		type(key, '  sk-ant-1 ');
		expect(cont.disabled).toBe(false);
		cont.click();
		flushSync();
		expect(onsubmit).toHaveBeenCalledWith({ provider_id: 'anthropic', api_key: 'sk-ant-1' });
	});

	it('goes back to the list, keeping the search', () => {
		const target = render();
		type(search(target), 'deep');
		pick(target, 'DeepSeek');
		expect(target.querySelector('[role="combobox"]')).toBeNull();
		buttonNamed(target, 'Back to providers')!.click();
		flushSync();
		expect(search(target).value).toBe('deep');
		expect(optionNames(target)).toEqual(['DeepSeek']);
	});

	it('opens straight on a provider when given one', () => {
		const target = render({ initialProviderId: 'openai' });
		expect(target.querySelector('h2')?.textContent).toBe('Connect OpenAI');
		expect(fieldLabelled(target, 'OpenAI API key')).toBeDefined();
	});

	it('asks a custom provider for a name, base URL, optional key and headers', () => {
		const onsubmit = vi.fn();
		const target = render({ onsubmit });
		pick(target, 'Custom OpenAI-compatible provider');
		const cont = buttonNamed(target, 'Continue')!;
		expect(cont.disabled).toBe(true);
		type(fieldLabelled(target, 'Name')!, 'Office vLLM');
		expect(cont.disabled).toBe(true);
		type(fieldLabelled(target, 'Base URL')!, 'https://llm.office.test/v1');
		expect(cont.disabled).toBe(false);
		expect(fieldLabelled(target, 'API key (optional)')).toBeDefined();

		buttonNamed(target, 'Add header')!.click();
		flushSync();
		buttonNamed(target, 'Add header')!.click();
		flushSync();
		let names = [...target.querySelectorAll<HTMLInputElement>('input[data-header="name"]')];
		let values = [...target.querySelectorAll<HTMLInputElement>('input[data-header="value"]')];
		expect(names).toHaveLength(2);
		type(names[0], 'X-Org');
		type(values[0], '42');
		type(names[1], 'X-Drop');
		buttonNamed(target, 'Remove this header')!.click();
		flushSync();
		names = [...target.querySelectorAll<HTMLInputElement>('input[data-header="name"]')];
		values = [...target.querySelectorAll<HTMLInputElement>('input[data-header="value"]')];
		expect(names).toHaveLength(1);
		// The first row went; what's left is the second one.
		expect(names[0].value).toBe('X-Drop');
		type(values[0], 'yes');
		buttonNamed(target, 'Add header')!.click();
		flushSync();

		cont.click();
		flushSync();
		expect(onsubmit).toHaveBeenCalledWith({
			provider_id: 'custom',
			label: 'Office vLLM',
			base_url: 'https://llm.office.test/v1',
			headers: [{ name: 'X-Drop', value: 'yes' }],
		});
	});

	it("shows the server's error and that it's busy", () => {
		const target = render({ initialProviderId: 'openai', error: "The provider didn't accept that key. Check it and try again.", busy: true });
		expect(target.querySelector('[role="alert"]')?.textContent).toContain("The provider didn't accept that key.");
		type(fieldLabelled(target, 'OpenAI API key')!, 'sk-1');
		const busy = buttonNamed(target, 'Connecting…')!;
		expect(busy.disabled).toBe(true);
	});
});
