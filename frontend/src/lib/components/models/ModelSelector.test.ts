// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import ModelSelector from './ModelSelector.svelte';
import type { ModelChoice, ModelGroup } from '$lib/modelPicker';

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
			{ key: 'conn:c1:meta/llama:free', label: 'Llama 3', model_id: 'meta/llama:free', free: true, enabled: true },
			{ key: 'conn:c1:openai/gpt-5', label: 'GPT-5', model_id: 'openai/gpt-5', free: false, enabled: true },
		],
	},
];

const sonnet: ModelChoice = {
	key: 'admin:a1',
	label: 'Claude Sonnet',
	model_id: 'claude-sonnet',
	provider_id: 'anthropic',
	provider_label: 'Anthropic',
	has_logo: true,
	chat_only: false,
};

let component: ReturnType<typeof mount> | null = null;

function render(props: { selected?: ModelChoice | null; onselect?: (key: string) => void; onmanage?: () => void } = {}) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(ModelSelector, {
		target,
		props: { groups, selected: sonnet, onselect: () => {}, onmanage: () => {}, ...props },
	});
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const trigger = (root: ParentNode) => root.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]')!;
const search = (root: ParentNode) => root.querySelector<HTMLInputElement>('input[role="combobox"]');
const optionNames = (root: ParentNode) => [...root.querySelectorAll('[role="option"]')].map((o) => o.querySelector('[data-name]')?.textContent?.trim());
const headers = (root: ParentNode) => [...root.querySelectorAll('[data-group-label]')].map((h) => h.textContent?.trim());

async function open(root: ParentNode) {
	trigger(root).click();
	flushSync();
	await tick();
	flushSync();
}

function type(input: HTMLInputElement, value: string) {
	input.value = value;
	input.dispatchEvent(new Event('input', { bubbles: true }));
	flushSync();
}

function press(input: HTMLElement, key: string) {
	const event = new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true });
	input.dispatchEvent(event);
	flushSync();
	return event;
}

describe('ModelSelector', () => {
	it("shows the selected model's logo and name on the trigger", () => {
		const target = render();
		expect(trigger(target).textContent).toContain('Claude Sonnet');
		expect(trigger(target).querySelector('[data-logo="anthropic"]')).not.toBeNull();
		expect(trigger(target).getAttribute('aria-label')).toBe('Model: Claude Sonnet');
		expect(search(target)).toBeNull();
	});

	it('opens on a focused search, models grouped under their providers', async () => {
		const target = render();
		await open(target);
		expect(document.activeElement).toBe(search(target));
		expect(headers(target)).toEqual(['Anthropic', 'OpenRouter']);
		// Haiku is switched off.
		expect(optionNames(target)).toEqual(['Claude Sonnet', 'Llama 3', 'GPT-5']);
		const free = [...target.querySelectorAll('[role="option"]')].filter((o) => o.textContent?.includes('Free'));
		expect(free).toHaveLength(1);
	});

	it('names each group for screen readers, even when its provider name has spaces', async () => {
		const spaced = [{ ...groups[0], id: 'nomi:Stub gateway', label: 'Stub gateway' }, groups[1]];
		const target = document.createElement('div');
		document.body.append(target);
		component = mount(ModelSelector, { target, props: { groups: spaced, selected: sonnet, onselect: () => {}, onmanage: () => {} } });
		flushSync();
		await open(target);
		const named = [...document.querySelectorAll('[role="group"][aria-labelledby]')].map((group) => {
			const ids = group.getAttribute('aria-labelledby')!.split(/\s+/);
			return ids.map((id) => document.getElementById(id)?.textContent?.trim() ?? null).join(' ');
		});
		expect(named).toEqual(['Stub gateway', 'OpenRouter']);
	});

	it('marks the selected model, and shows it even when switched off', async () => {
		const haiku: ModelChoice = { ...sonnet, key: 'admin:a2', label: 'Claude Haiku', model_id: 'claude-haiku' };
		const target = render({ selected: haiku });
		await open(target);
		expect(optionNames(target)).toEqual(['Claude Sonnet', 'Claude Haiku', 'Llama 3', 'GPT-5']);
		const selected = [...target.querySelectorAll('[role="option"][aria-selected="true"]')];
		expect(selected.map((o) => o.querySelector('[data-name]')?.textContent)).toEqual(['Claude Haiku']);
		expect(selected[0].querySelector('[data-check]')).not.toBeNull();
	});

	it('filters as the person types, and says when nothing matches', async () => {
		const target = render();
		await open(target);
		type(search(target)!, 'gpt');
		expect(optionNames(target)).toEqual(['GPT-5']);
		expect(headers(target)).toEqual(['OpenRouter']);
		type(search(target)!, 'zzz');
		expect(optionNames(target)).toEqual([]);
		expect(target.textContent).toContain('Nothing matches “zzz”.');
	});

	it('walks the rows with the arrow keys, wrapping, and picks with Enter', async () => {
		const onselect = vi.fn();
		const target = render({ onselect });
		await open(target);
		const input = search(target)!;
		const active = () => document.getElementById(input.getAttribute('aria-activedescendant') ?? '')?.querySelector('[data-name]')?.textContent;
		// Opens on the selected row.
		expect(active()).toBe('Claude Sonnet');
		press(input, 'ArrowDown');
		expect(active()).toBe('Llama 3');
		press(input, 'ArrowDown');
		press(input, 'ArrowDown');
		expect(active()).toBe('Claude Sonnet');
		press(input, 'ArrowUp');
		expect(active()).toBe('GPT-5');
		const enter = press(input, 'Enter');
		expect(enter.defaultPrevented).toBe(true);
		expect(onselect).toHaveBeenCalledWith('conn:c1:openai/gpt-5');
		await tick();
		flushSync();
		expect(search(target)).toBeNull();
	});

	it('picks with a click, and not again when it is already the one', async () => {
		const onselect = vi.fn();
		const target = render({ onselect });
		await open(target);
		[...target.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes('Llama 3'))!.click();
		flushSync();
		expect(onselect).toHaveBeenCalledWith('conn:c1:meta/llama:free');
		await open(target);
		[...target.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes('Claude Sonnet'))!.click();
		flushSync();
		expect(onselect).toHaveBeenCalledTimes(1);
	});

	it('closes on Escape', async () => {
		const target = render();
		await open(target);
		press(search(target)!, 'Escape');
		await tick();
		flushSync();
		expect(search(target)).toBeNull();
		expect(trigger(target).getAttribute('aria-expanded')).toBe('false');
	});

	it('opens Manage models from the bottom row', async () => {
		const onmanage = vi.fn();
		const target = render({ onmanage });
		await open(target);
		const manage = [...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Manage models')!;
		manage.click();
		flushSync();
		expect(onmanage).toHaveBeenCalled();
		expect(search(target)).toBeNull();
	});

	it('asks to choose when no model is known', () => {
		const target = render({ selected: null });
		expect(trigger(target).textContent).toContain('Choose model');
	});
});
