// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, flushSync, mount, tick, unmount, type Component } from 'svelte';
import { House, Pencil, Trash2 } from '@lucide/svelte';
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
import NavItem from './navigation/NavItem.svelte';
import RecentsList from './chrome/RecentsList.svelte';
import Sidebar from './chrome/Sidebar.svelte';
import SettingsNav from './chrome/SettingsNav.svelte';
import UsageCard from './chrome/UsageCard.svelte';
import AppShell from './shells/AppShell.svelte';
import WorkspaceShell from './shells/WorkspaceShell.svelte';
import SkeletonRow from './lists/SkeletonRow.svelte';
import SuggestionRow from './lists/SuggestionRow.svelte';
import ProjectRow from './lists/ProjectRow.svelte';

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

// Each test mounts a different twin with its own props; the props are checked by the twin itself.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, props: Record<string, unknown>): HTMLElement {
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

const html = (markup: string) => createRawSnippet(() => ({ render: () => markup }));

describe('navigation and chrome', () => {
	it('NavItem marks the page it leads to', () => {
		expect(render(NavItem, { href: '/', icon: House, label: 'Home', active: true }).querySelector('a')!.getAttribute('aria-current')).toBe('page');
		expect(render(NavItem, { href: '/chats', icon: House, label: 'Chats', count: 3 }).querySelector('a')!.getAttribute('aria-current')).toBeNull();
	});

	it('RecentsList folds its rows but keeps the way to every chat', () => {
		const items = [{ href: '/chat/a', kind: 'chat', title: 'Trip', when: '2h' }];
		const root = render(RecentsList, { items, allHref: '/chats' });
		const fold = root.querySelector<HTMLButtonElement>('button[aria-expanded]')!;
		expect(root.querySelector('a[href="/chat/a"]')).not.toBeNull();
		fold.click();
		flushSync();
		expect(fold.getAttribute('aria-expanded')).toBe('false');
		expect(root.querySelector('a[href="/chat/a"]')).toBeNull();
		expect(root.querySelector('a[href="/chats"]')).not.toBeNull();
	});

	it('Sidebar is the main navigation, with New chat and the sections it is given', () => {
		const oncollapse = vi.fn();
		const root = render(Sidebar, { newChatHref: '/chat/new', oncollapse, children: html('<a href="/money">Money</a>') });
		expect(root.querySelector('nav')!.getAttribute('aria-label')).toBe('Main');
		expect(root.querySelector('a[href="/chat/new"]')?.textContent).toContain('⌘K');
		expect(root.querySelector('a[href="/money"]')).not.toBeNull();
		root.querySelector<HTMLButtonElement>('button[aria-label]')!.click();
		expect(oncollapse).toHaveBeenCalledOnce();
	});

	it('SettingsNav marks the open setting and logs out', () => {
		const onlogout = vi.fn();
		const groups = [{ label: 'Account', items: [{ href: '/profile', icon: 'user', label: 'Profile' }, { href: '/billing', icon: 'credit-card', label: 'Billing' }] }];
		const root = render(SettingsNav, { backHref: '/', groups, active: '/billing', user: { name: 'Ada' }, version: 'v1.2', onlogout });
		expect(root.querySelector('[aria-current="page"]')?.getAttribute('href')).toBe('/billing');
		const logout = [...root.querySelectorAll('button')].find((button) => button.textContent?.includes('v1.2'))!;
		logout.click();
		expect(onlogout).toHaveBeenCalledOnce();
	});

	it('UsageCard shows the plan as a meter', () => {
		const meter = render(UsageCard, { plan: 'Pro', used: 0.25, caption: '250K / 1M', href: '/billing' }).querySelector('[role="meter"]')!;
		expect(meter.getAttribute('aria-valuenow')).toBe('25');
	});
});

describe('shells and lists', () => {
	it('AppShell opens the navigation as a modal drawer on small screens', () => {
		const root = render(AppShell, { sidebar: html('<nav>Drawer</nav>'), children: html('<p>Content</p>') });
		expect(root.querySelector('dialog')).toBeNull();
		root.querySelector<HTMLButtonElement>('header button')!.click();
		flushSync();
		expect(root.querySelector('dialog')?.textContent).toContain('Drawer');
	});

	it('WorkspaceShell switches between chat and preview', () => {
		const root = render(WorkspaceShell, { projectbar: html('<span>Todo</span>'), chat: html('<p>Chat</p>'), children: html('<p>Preview</p>') });
		const group = root.querySelector('[role="radiogroup"]')!;
		expect(root.querySelector('section')!.classList.contains('shown')).toBe(true);
		key(group, 'ArrowRight');
		expect(root.querySelector('section')!.classList.contains('shown')).toBe(false);
	});

	it('SkeletonRow says what is loading', () => {
		const status = render(SkeletonRow, { label: 'loading 4 projects…' }).querySelector('[role="status"]')!;
		expect(status.getAttribute('aria-busy')).toBe('true');
		expect(status.textContent).toContain('loading 4 projects…');
	});

	it('SuggestionRow asks when picked', () => {
		const onclick = vi.fn();
		render(SuggestionRow, { agent: 'dana', label: 'How much did I spend?', route: '→ dana', onclick }).querySelector('button')!.click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('ProjectRow links to the project with its status in words', () => {
		const root = render(ProjectRow, { href: '/projects/1', name: 'todo', tone: 'koda', status: { kind: 'live', label: 'Live' }, url: 'todo.nomi.app', updated: '2h' });
		expect(root.querySelector('a')!.getAttribute('href')).toBe('/projects/1');
		expect(root.textContent).toContain('Live');
	});
});
