// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';
import ToolPicker from './ToolPicker.svelte';
import SoulEditor from './SoulEditor.svelte';
import type { CrewRosterMember } from '$lib/crew';
import type { CatalogTool } from '$lib/types';

const catalog: CatalogTool[] = [
	{ name: 'list_files', description: 'List project files', group: 'Coding', keywords: ['browse'] },
	{ name: 'read_file', description: 'Read a project file', group: 'Coding', keywords: [] },
	{ name: 'set_budget', description: 'Set a budget', group: 'Money', keywords: ['spending'] },
];
const roster = [{ agent_type: 'money', name: 'Dana', role: 'Money' }] as CrewRosterMember[];

let component: ReturnType<typeof mount> | null = null;
afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mountInto(Component: Component<any, any, any>, props: Record<string, unknown>) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(Component, { target, props });
	flushSync();
	return target;
}

function press(el: Element, key: string) {
	el.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
	flushSync();
}

function typeInto(el: HTMLInputElement | HTMLTextAreaElement, value: string) {
	el.value = value;
	if (el instanceof HTMLTextAreaElement) el.setSelectionRange(value.length, value.length);
	el.dispatchEvent(new Event('input', { bubbles: true }));
	el.dispatchEvent(new KeyboardEvent('keyup', { key: value.slice(-1), bubbles: true }));
	flushSync();
}

describe('ToolPicker', () => {
	function picker(selected: string[] = []) {
		const state = { selected };
		const target = mountInto(ToolPicker, {
			options: catalog,
			label: 'Tools',
			name: 'tools',
			get selected() {
				return state.selected;
			},
			set selected(value: string[]) {
				state.selected = value;
			},
		});
		return { target, state, input: target.querySelector('input[role="combobox"]') as HTMLInputElement };
	}

	it('is a combobox over a multiselectable listbox, filtered as you type', () => {
		const { target, input } = picker();
		input.focus();
		input.dispatchEvent(new FocusEvent('focus'));
		flushSync();
		expect(input.getAttribute('aria-expanded')).toBe('true');
		expect(target.querySelector('[role="listbox"]')?.getAttribute('aria-multiselectable')).toBe('true');
		expect(target.querySelectorAll('[role="option"]')).toHaveLength(3);
		typeInto(input, 'browse');
		expect([...target.querySelectorAll('[role="option"]')].map((o) => o.textContent)).toEqual([expect.stringContaining('list_files')]);
		expect(input.getAttribute('aria-activedescendant')).toBe(target.querySelector('[role="option"]')?.id);
	});

	it('picks with the arrows and Enter, then Backspace in an empty field removes the last chip', () => {
		const { state, input } = picker();
		input.dispatchEvent(new FocusEvent('focus'));
		flushSync();
		press(input, 'ArrowDown');
		press(input, 'Enter');
		expect(state.selected).toEqual(['read_file']);
		typeInto(input, 'budget');
		press(input, 'Enter');
		expect(state.selected).toEqual(['read_file', 'set_budget']);
		typeInto(input, '');
		press(input, 'Backspace');
		expect(state.selected).toEqual(['read_file']);
	});

	it('Escape closes the list and a chip removes its tool', () => {
		const { target, state, input } = picker(['list_files']);
		expect([...target.querySelectorAll('input[type="hidden"]')].map((i) => (i as HTMLInputElement).name + '=' + (i as HTMLInputElement).value)).toEqual(['tools=list_files']);
		input.dispatchEvent(new FocusEvent('focus'));
		flushSync();
		press(input, 'Escape');
		expect(input.getAttribute('aria-expanded')).toBe('false');
		(target.querySelector('button[aria-label]') as HTMLButtonElement).click();
		flushSync();
		expect(state.selected).toEqual([]);
	});
});

describe('SoulEditor', () => {
	it('offers the crew then the tools after "@" and tells the page which tool was mentioned', async () => {
		const ontoolmention = vi.fn();
		const state = { value: '' };
		const target = mountInto(SoulEditor, {
			name: 'soul',
			label: 'soul.md',
			crew: roster,
			tools: catalog,
			ontoolmention,
			get value() {
				return state.value;
			},
			set value(next: string) {
				state.value = next;
			},
		});
		const field = target.querySelector('textarea') as HTMLTextAreaElement;
		typeInto(field, 'Use @');
		await tick();
		const options = [...target.querySelectorAll('[role="option"]')].map((o) => o.textContent?.trim() ?? '');
		expect(options[0]).toContain('Dana');
		expect(options[1]).toContain('list_files');
		typeInto(field, 'Use @read');
		await tick();
		press(field, 'Enter');
		await tick();
		expect(state.value).toBe('Use @read_file ');
		expect(ontoolmention).toHaveBeenCalledWith('read_file');
	});

	it('does not report a crew mention as a tool', async () => {
		const ontoolmention = vi.fn();
		const target = mountInto(SoulEditor, { value: '', name: 'soul', label: 'soul.md', crew: roster, tools: catalog, ontoolmention });
		const field = target.querySelector('textarea') as HTMLTextAreaElement;
		typeInto(field, '@Da');
		await tick();
		press(field, 'Enter');
		await tick();
		expect(field.value).toBe('@Dana ');
		expect(ontoolmention).not.toHaveBeenCalled();
	});
});
