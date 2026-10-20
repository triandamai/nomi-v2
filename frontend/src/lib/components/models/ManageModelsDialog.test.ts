// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import ManageModelsDialog from './ManageModelsDialog.svelte';
import type { ModelChange, ModelGroup } from '$lib/modelPicker';

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

const groups: ModelGroup[] = [
	{
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
	},
	{
		id: 'conn:c1',
		source: 'connection',
		provider_id: 'openrouter',
		has_logo: true,
		label: 'OpenRouter',
		connection_id: 'c1',
		error: null,
		models: [
			{ key: 'conn:c1:meta/llama:free', label: 'Llama 3', model_id: 'meta/llama:free', free: true, enabled: false },
			{ key: 'conn:c1:openai/gpt-5', label: 'GPT-5', model_id: 'openai/gpt-5', free: false, enabled: false },
		],
	},
	{
		id: 'conn:c2',
		source: 'connection',
		provider_id: 'custom',
		has_logo: false,
		label: 'Office LLM',
		connection_id: 'c2',
		error: 'unreachable',
		models: [],
	},
];

type Props = { groups: ModelGroup[]; onchange: (changes: ModelChange[]) => void | Promise<boolean>; onconnect: () => void };

let component: ReturnType<typeof mount> | null = null;

function render(props: Partial<Props> = {}) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ManageModelsDialog, { target, props: { open: true, groups, onchange: () => {}, onconnect: () => {}, ...props } });
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const buttonNamed = (root: ParentNode, text: string) =>
	[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === text || b.getAttribute('aria-label') === text);
const switchNamed = (root: ParentNode, name: string) => root.querySelector<HTMLButtonElement>(`[role="switch"][aria-label="${name}"]`);
const modelNames = (root: ParentNode) => [...root.querySelectorAll('[data-model-name]')].map((n) => n.textContent?.trim());
const groupNames = (root: ParentNode) => [...root.querySelectorAll('[data-group-name]')].map((n) => n.textContent?.trim());
const settle = async () => {
	for (let i = 0; i < 5; i++) await tick();
	flushSync();
};

function type(input: HTMLInputElement, value: string) {
	input.value = value;
	input.dispatchEvent(new Event('input', { bubbles: true }));
	flushSync();
}

describe('ManageModelsDialog', () => {
	it('says what it is for and connects a provider from the header', () => {
		const onconnect = vi.fn();
		const target = render({ onconnect });
		expect(target.querySelector('h2')?.textContent).toBe('Manage models');
		expect(target.textContent).toContain('Customize which models appear in the model selector.');
		buttonNamed(target, 'Connect provider')!.click();
		flushSync();
		expect(onconnect).toHaveBeenCalled();
	});

	it('lists every provider and its models, with a quiet note for one that could not be reached', () => {
		const target = render();
		expect(groupNames(target)).toEqual(['Anthropic', 'OpenRouter', 'Office LLM']);
		expect(modelNames(target)).toEqual(['Claude Sonnet', 'Claude Haiku', 'Llama 3', 'GPT-5']);
		expect(target.textContent).toContain("Couldn't load this provider's models right now.");
		expect(target.textContent).toContain('Free');
		// A provider switch shows on while any of its models is.
		expect(switchNamed(target, 'Show all Anthropic models')?.getAttribute('aria-checked')).toBe('true');
		expect(switchNamed(target, 'Show all OpenRouter models')?.getAttribute('aria-checked')).toBe('false');
	});

	it('names each provider section for screen readers, even when the name has spaces', () => {
		render({ groups: groups.map((group, i) => (i === 0 ? { ...group, id: 'nomi:Stub gateway', label: 'Stub gateway' } : group)) });
		const sections = [...document.querySelectorAll('section[aria-labelledby]')];
		const named = sections.map((section) =>
			section.getAttribute('aria-labelledby')!.split(/\s+/).map((id) => document.getElementById(id)?.textContent?.trim() ?? null).join(' '),
		);
		expect(named[0]).toBe('Stub gateway');
	});

	it('filters models and providers as the person types', () => {
		const target = render();
		const search = target.querySelector<HTMLInputElement>('input[type="search"], input[type="text"]')!;
		expect(search.placeholder).toBe('Search models');
		type(search, 'gpt');
		expect(modelNames(target)).toEqual(['GPT-5']);
		expect(groupNames(target)).toEqual(['OpenRouter']);
		type(search, 'anthropic');
		expect(modelNames(target)).toEqual(['Claude Sonnet', 'Claude Haiku']);
		type(search, 'zzz');
		expect(target.textContent).toContain('Nothing matches “zzz”.');
	});

	it("turns a whole provider on or off with its switch, in one change", async () => {
		const onchange = vi.fn(async () => true);
		const target = render({ onchange });
		switchNamed(target, 'Show all OpenRouter models')!.click();
		await settle();
		expect(onchange).toHaveBeenCalledWith([
			{ key: 'conn:c1:meta/llama:free', enabled: true },
			{ key: 'conn:c1:openai/gpt-5', enabled: true },
		]);
		expect(switchNamed(target, 'Show GPT-5 in the model selector')?.getAttribute('aria-checked')).toBe('true');

		// Mixed shows on, so the switch turns them all off.
		switchNamed(target, 'Show all Anthropic models')!.click();
		await settle();
		expect(onchange).toHaveBeenLastCalledWith([
			{ key: 'admin:a1', enabled: false },
			{ key: 'admin:a2', enabled: false },
		]);
		expect(switchNamed(target, 'Show all Anthropic models')?.getAttribute('aria-checked')).toBe('false');
	});

	it('switches one model', async () => {
		const onchange = vi.fn(async () => true);
		const target = render({ onchange });
		switchNamed(target, 'Show Claude Haiku in the model selector')!.click();
		await settle();
		expect(onchange).toHaveBeenCalledWith([{ key: 'admin:a2', enabled: true }]);
		expect(switchNamed(target, 'Show Claude Haiku in the model selector')?.getAttribute('aria-checked')).toBe('true');
		expect(switchNamed(target, 'Show all Anthropic models')?.getAttribute('aria-checked')).toBe('true');
	});

	it('puts a switch back and says so when saving fails', async () => {
		const onchange = vi.fn(async () => false);
		const target = render({ onchange });
		switchNamed(target, 'Show Claude Sonnet in the model selector')!.click();
		await settle();
		expect(onchange).toHaveBeenCalledWith([{ key: 'admin:a1', enabled: false }]);
		expect(switchNamed(target, 'Show Claude Sonnet in the model selector')?.getAttribute('aria-checked')).toBe('true');
		expect(target.querySelector('[role="alert"]')?.textContent).toContain("Couldn't save your models. Try again.");
	});

	it('closes from its close button', () => {
		const target = render();
		buttonNamed(target, 'Close')!.click();
		flushSync();
		expect(target.querySelector('dialog')?.hasAttribute('open')).toBe(false);
	});
});
