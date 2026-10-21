// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import EffortMenu from './EffortMenu.svelte';
import type { ThinkingLevel } from '$lib/types';

// jsdom has no popover API or ResizeObserver; the menu only needs them to exist.
beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLElement.prototype.showPopover ??= function () {};
	HTMLElement.prototype.hidePopover ??= function () {};
});

let component: ReturnType<typeof mount> | null = null;

function render(level: ThinkingLevel, onchange: (level: ThinkingLevel) => void = () => {}) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(EffortMenu, { target, props: { level, onchange } });
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const options = (root: ParentNode) => [...root.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')];
const trigger = (root: ParentNode) => root.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"]')!;

describe('EffortMenu', () => {
	it('offers Default, Low, High and Max in that order, and no Off', () => {
		const root = render('medium');
		expect(options(root).map((o) => o.textContent?.trim())).toEqual(['Default', 'Low', 'High', 'Max']);
		expect(root.textContent).not.toMatch(/\bOff\b/);
	});

	it('checks the active option only', () => {
		const root = render('high');
		const checked = options(root).filter((o) => o.getAttribute('aria-checked') === 'true');
		expect(checked.map((o) => o.textContent?.trim())).toEqual(['High']);
		expect(checked[0].querySelector('[data-check]')).not.toBeNull();
		expect(options(root).filter((o) => o.querySelector('[data-check]'))).toHaveLength(1);
	});

	it('shows the current label on the trigger, with medium as Default', () => {
		expect(trigger(render('medium')).textContent?.trim()).toBe('Default');
		unmount(component!);
		document.body.innerHTML = '';
		expect(trigger(render('max')).textContent?.trim()).toBe('Max');
	});

	it.each([
		['Default', 'medium'],
		['Low', 'low'],
		['High', 'high'],
		['Max', 'max'],
	] as const)('picking %s sends %s', async (label, level) => {
		const onchange = vi.fn();
		// Start somewhere else so every pick is a change.
		const root = render(level === 'low' ? 'high' : 'low', onchange);
		trigger(root).click();
		flushSync();
		await tick();
		options(root)
			.find((o) => o.textContent?.trim() === label)!
			.click();
		flushSync();
		expect(onchange).toHaveBeenCalledWith(level);
	});

	it('does not report picking the level that is already on', () => {
		const onchange = vi.fn();
		const root = render('low', onchange);
		options(root)
			.find((o) => o.textContent?.trim() === 'Low')!
			.click();
		expect(onchange).not.toHaveBeenCalled();
	});
});
