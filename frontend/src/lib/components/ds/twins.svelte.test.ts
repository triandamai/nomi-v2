// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, flushSync, mount, tick, unmount } from 'svelte';
import { Pencil, Trash2 } from '@lucide/svelte';
import Button from './actions/Button.svelte';
import IconButton from './actions/IconButton.svelte';
import Meter from './status/Meter.svelte';
import Status from './status/Status.svelte';
import Avatar from './identity/Avatar.svelte';
import Mascot from './agents/Mascot.svelte';
import Checkbox from './inputs/Checkbox.svelte';
import Toggle from './inputs/Toggle.svelte';
import Segmented from './inputs/Segmented.svelte';
import Tabs from './inputs/Tabs.svelte';
import TextField from './inputs/TextField.svelte';
import Chip from './inputs/Chip.svelte';
import Dialog from './overlays/Dialog.svelte';
import Menu from './overlays/Menu.svelte';
import Toast from './overlays/Toast.svelte';
import ToastRegion from './overlays/ToastRegion.svelte';
import { toasts } from './overlays/toasts.svelte';

// The twins behave like frontend-new's design system: the same roles, keys and events. Their look
// is Material 3, so nothing here checks styling.

// jsdom has no popover API, ResizeObserver or modal dialogs; the components only need them to exist.
beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLElement.prototype.showPopover ??= function () {};
	HTMLElement.prototype.hidePopover ??= function () {};
	HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
		this.open = true;
	};
});

let mounted: ReturnType<typeof mount>[] = [];

function render<Props extends Record<string, unknown>>(component: Parameters<typeof mount>[0], props: Props): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props }));
	flushSync();
	return target;
}

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	toasts.clear();
	document.body.innerHTML = '';
	vi.useRealTimers();
});

const key = (element: Element, name: string) => {
	element.dispatchEvent(new KeyboardEvent('keydown', { key: name, bubbles: true }));
	flushSync();
};

describe('actions', () => {
	it('Button is a button until it has an href, then a link that opens other sites safely', () => {
		const onclick = vi.fn();
		const button = render(Button, { label: 'Save', onclick }).querySelector('button')!;
		button.click();
		expect(onclick).toHaveBeenCalledOnce();

		const link = render(Button, { label: 'Docs', href: 'https://example.com', external: true }).querySelector('a')!;
		expect(link.getAttribute('target')).toBe('_blank');
		expect(link.getAttribute('rel')).toBe('noopener noreferrer');
	});

	it('Button is busy and not clickable while loading', () => {
		const button = render(Button, { label: 'Deploying · 00:14', loading: true }).querySelector('button')!;
		expect(button.disabled).toBe(true);
		expect(button.getAttribute('aria-busy')).toBe('true');
	});

	it('IconButton is named by its label', () => {
		const button = render(IconButton, { icon: Pencil, label: 'Rename' }).querySelector('button')!;
		expect(button.getAttribute('aria-label')).toBe('Rename');
	});
});

describe('status and identity', () => {
	it('Meter reports the real figure, never more than full', () => {
		const meter = render(Meter, { label: 'Tokens', value: 1.4, caption: '1.4M / 1M' }).querySelector('[role="meter"]')!;
		expect(meter.getAttribute('aria-valuenow')).toBe('100');
		expect(meter.getAttribute('aria-valuetext')).toBe('100% · 1.4M / 1M');
	});

	it('Status shows who is working instead of a dot', () => {
		expect(render(Status, { kind: 'live', label: 'Live' }).querySelector('[data-dot]')).not.toBeNull();
		const working = render(Status, { kind: 'working', label: 'Building', agent: 'koda' });
		expect(working.querySelector('[data-dot]')).toBeNull();
		expect(working.querySelector('svg')).not.toBeNull();
	});

	it('Avatar falls back to initials named for screen readers', () => {
		const avatar = render(Avatar, { name: 'ada lovelace byron' }).querySelector('[data-avatar]')!;
		expect(avatar.getAttribute('aria-label')).toBe('ada lovelace byron');
		expect(avatar.textContent?.trim()).toBe('AL');
	});

	it('Mascot is decorative unless it stands alone', () => {
		expect(render(Mascot, { agent: 'dana', size: 20 }).querySelector('svg')!.getAttribute('role')).toBeNull();
		expect(render(Mascot, { agent: 'dana', size: 32, label: 'Dana' }).querySelector('svg')!.getAttribute('aria-label')).toBe('Dana');
	});
});

describe('inputs', () => {
	it('Checkbox and Toggle report the new state', () => {
		const checked = vi.fn();
		render(Checkbox, { label: 'Remember', onchange: checked }).querySelector('input')!.click();
		expect(checked).toHaveBeenCalledWith(true);

		const flipped = vi.fn();
		const toggle = render(Toggle, { label: 'Notifications', onchange: flipped }).querySelector('[role="switch"]')!;
		expect(toggle.getAttribute('aria-label')).toBe('Notifications');
		(toggle as HTMLButtonElement).click();
		flushSync();
		expect(flipped).toHaveBeenCalledWith(true);
		expect(toggle.getAttribute('aria-checked')).toBe('true');
	});

	it('Segmented is a radio group the arrow keys move through, wrapping', () => {
		const onchange = vi.fn();
		const options = [
			{ value: 'all', label: 'All', count: 4 },
			{ value: 'live', label: 'Live', count: 2 },
		];
		const group = render(Segmented, { label: 'Filter', options, onchange }).querySelector('[role="radiogroup"]')!;
		key(group, 'ArrowRight');
		expect(onchange).toHaveBeenLastCalledWith('live');
		key(group, 'ArrowRight');
		expect(onchange).toHaveBeenLastCalledWith('all');
		expect(group.querySelector('[aria-checked="true"]')?.textContent).toContain('All');
	});

	it('Tabs open with the arrow keys, Home and End, and label their panel', () => {
		const onchange = vi.fn();
		const tabs = [
			{ value: 'code', label: 'Code' },
			{ value: 'preview', label: 'Preview' },
			{ value: 'logs', label: 'Logs' },
		];
		const panel = createRawSnippet((value: () => string) => ({ render: () => `<p>${value()}</p>` }));
		const root = render(Tabs, { label: 'Views', tabs, onchange, panel });
		const list = root.querySelector('[role="tablist"]')!;
		key(list, 'End');
		expect(onchange).toHaveBeenLastCalledWith('logs');
		key(list, 'Home');
		key(list, 'ArrowLeft');
		expect(onchange).toHaveBeenLastCalledWith('logs');
		const selected = root.querySelector('[role="tab"][aria-selected="true"]')!;
		expect(root.querySelector('[role="tabpanel"]')!.getAttribute('aria-labelledby')).toBe(selected.id);
	});

	it('TextField says what went wrong in place of the helper', () => {
		const root = render(TextField, { label: 'Slug', helper: 'Lowercase', error: 'Already taken' });
		const input = root.querySelector('input')!;
		expect(input.getAttribute('aria-invalid')).toBe('true');
		expect(document.getElementById(input.getAttribute('aria-describedby')!)?.textContent).toBe('Already taken');
		expect(root.textContent).not.toContain('Lowercase');
	});

	it('Chip opens a menu, except a provision, which only names what was given', () => {
		expect(render(Chip, { kind: 'model', label: 'gpt-5' }).querySelector('button')!.getAttribute('aria-haspopup')).toBe('menu');
		expect(render(Chip, { kind: 'provision', label: 'postgres' }).querySelector('button')).toBeNull();
	});
});

describe('overlays', () => {
	it('Dialog opens as a modal named by its title and reports closing', async () => {
		const onclose = vi.fn();
		const target = document.body.appendChild(document.createElement('div'));
		const props = $state({ open: true, title: 'Delete chat?', description: 'This can’t be undone.', onclose });
		mounted.push(mount(Dialog, { target, props }));
		flushSync();
		const dialog = target.querySelector('dialog')!;
		expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('Delete chat?');
		props.open = false;
		flushSync();
		await tick();
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('Menu opens from its trigger, moves with the arrows and picks with a click', async () => {
		const edit = vi.fn();
		const remove = vi.fn();
		const trigger = createRawSnippet((props: () => { onclick: () => void; 'aria-expanded': boolean }) => ({
			render: () => '<button>Open</button>',
			setup: (button) => {
				button.addEventListener('click', () => props().onclick());
				$effect(() => button.setAttribute('aria-expanded', String(props()['aria-expanded'])));
			},
		}));
		const root = render(Menu, {
			trigger,
			items: [{ label: 'Rename', icon: Pencil, onselect: edit }, 'divider', { label: 'Delete', icon: Trash2, danger: true, onselect: remove }],
		});
		root.querySelector('button')!.click();
		flushSync();
		await tick();
		const items = [...root.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')];
		expect(items.map((item) => item.textContent?.trim())).toEqual(['Rename', 'Delete']);
		expect(document.activeElement).toBe(items[0]);
		key(root.querySelector('[role="menu"]')!, 'ArrowDown');
		expect(document.activeElement).toBe(items[1]);
		items[1].click();
		flushSync();
		expect(remove).toHaveBeenCalledOnce();
		expect(root.querySelector('[role="menu"]')).toBeNull();
	});

	it('Toast interrupts only for errors', () => {
		expect(render(Toast, { kind: 'error', title: 'Deploy failed' }).querySelector('[role="alert"]')).not.toBeNull();
		expect(render(Toast, { kind: 'success', title: 'Saved' }).querySelector('[role="status"]')).not.toBeNull();
	});

	it('toasts go away on their own, except errors', () => {
		vi.useFakeTimers();
		const root = render(ToastRegion, {});
		toasts.show({ kind: 'success', title: 'Saved' });
		toasts.show({ kind: 'error', title: 'Deploy failed' });
		flushSync();
		expect(root.querySelectorAll('[role="status"], [role="alert"]')).toHaveLength(2);
		vi.advanceTimersByTime(6000);
		flushSync();
		expect([...root.querySelectorAll('[role="status"], [role="alert"]')].map((toast) => toast.getAttribute('role'))).toEqual(['alert']);
	});
});
