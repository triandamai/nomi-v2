// @vitest-environment jsdom
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';

// The app's frame on the design-system twins: the desktop sidebar is the twin drawer, an icon rail
// when collapsed, and the settings drawer on an account page; the phone's drawer is the same.

const page = vi.hoisted(() => ({ url: new URL('http://localhost/'), params: {}, data: {} }));
const goto = vi.hoisted(() => vi.fn());
vi.mock('$app/state', () => ({ page, navigating: { to: null }, updated: { current: false } }));
vi.mock('$app/navigation', () => ({ goto, invalidateAll: async () => {}, afterNavigate: () => {}, beforeNavigate: () => {} }));

const { default: Sidebar } = await import('./Sidebar.svelte');
const { default: MobileNavDrawer } = await import('./MobileNavDrawer.svelte');

beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLElement.prototype.showPopover ??= function () {};
	HTMLElement.prototype.hidePopover ??= function () {};
	// A desktop-sized window: the sidebar starts expanded unless it was collapsed.
	window.matchMedia ??= (query: string) => ({ matches: false, media: query, addEventListener() {}, removeEventListener() {} }) as unknown as MediaQueryList;
});

const props = {
	userEmail: 'ada@nomi.test',
	profile: { display_name: 'Ada Lovelace', avatar_url: null },
	usage: null,
	savedPins: ['notifications'],
	recents: { enabled: true, limit: 10 },
};

let mounted: ReturnType<typeof mount>[] = [];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, extra: Record<string, unknown> = {}): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props: { ...props, ...extra } }));
	flushSync();
	return target;
}

const labelled = (root: HTMLElement, label: string) => root.querySelector<HTMLElement>(`[aria-label="${label}"]`);

beforeEach(() => {
	globalThis.fetch = vi.fn(async () => new Response('[]', { headers: { 'content-type': 'application/json' } }));
	page.url = new URL('http://localhost/chats');
	localStorage.clear();
});

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	document.body.innerHTML = '';
	goto.mockReset();
});

describe('desktop sidebar', () => {
	it('is the twin drawer: New chat, the places with the open one marked, and the person', () => {
		const root = render(Sidebar);
		const nav = labelled(root, 'Main navigation') ?? root.querySelector('nav')!;
		expect(nav.querySelector('a[href="/chat/new"]')).not.toBeNull();
		expect(root.querySelector('a[href="/chats"]')!.getAttribute('aria-current')).toBe('page');
		expect(root.querySelector('a[href="/notifications"]')).not.toBeNull();
		expect(root.querySelector('a[href="/preferences"]')).not.toBeNull();
		expect(root.textContent).toContain('Ada Lovelace');
	});

	it('collapses to an icon rail of the same places, and expands again', async () => {
		const root = render(Sidebar);
		labelled(root, 'Collapse sidebar')!.click();
		flushSync();
		const rail = root.querySelector('.rail')!;
		expect(rail).not.toBeNull();
		expect(rail.querySelector('a[href="/chats"]')!.getAttribute('aria-current')).toBe('page');
		expect(rail.querySelector('a[aria-label="Home"]')!.getAttribute('href')).toBe('/');
		expect(localStorage.getItem('nomi:user-sidebar-collapsed')).toBe('true');
		labelled(root, 'Expand sidebar')!.click();
		flushSync();
		await tick();
		expect(root.querySelector('.rail')).toBeNull();
		expect(localStorage.getItem('nomi:user-sidebar-collapsed')).toBe('false');
	});

	it('remembers being collapsed', async () => {
		localStorage.setItem('nomi:user-sidebar-collapsed', 'true');
		const root = render(Sidebar);
		await tick();
		expect(root.querySelector('.rail')).not.toBeNull();
	});
});

describe('settings drawer', () => {
	it('takes the place of the navigation on an account page, the open one marked', () => {
		page.url = new URL('http://localhost/billing');
		const root = render(Sidebar);
		expect(root.querySelector('a[href="/"]')!.textContent).toContain('Back');
		expect(root.querySelector('a[href="/billing"]')!.getAttribute('aria-current')).toBe('page');
		expect(root.querySelector('a[href="/memory"]')).not.toBeNull();
		expect(root.querySelector('a[href="/chats"]')).toBeNull();
	});

	it('searches the settings', () => {
		page.url = new URL('http://localhost/profile');
		const root = render(Sidebar);
		const search = root.querySelector<HTMLInputElement>('input')!;
		search.value = 'bill';
		search.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		expect(root.querySelector('a[href="/billing"]')).not.toBeNull();
		expect(root.querySelector('a[href="/profile"]')).toBeNull();
	});

	it('goes back to Nomi on Esc, but not while typing', () => {
		page.url = new URL('http://localhost/models');
		const root = render(Sidebar);
		root.querySelector('input')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
		expect(goto).not.toHaveBeenCalled();
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
		expect(goto).toHaveBeenCalledWith('/');
	});

	it('logs out through the logout form', () => {
		page.url = new URL('http://localhost/account');
		const submit = vi.fn();
		HTMLFormElement.prototype.requestSubmit = submit;
		const root = render(Sidebar);
		[...root.querySelectorAll('button')].find((button) => button.textContent?.includes('Log out'))!.click();
		expect(submit).toHaveBeenCalledOnce();
		expect(root.querySelector('form[action="/logout"]')!.getAttribute('method')).toBe('POST');
	});
});

describe('phone drawer', () => {
	it('shows the same drawer, or the settings one on an account page, and closes on Esc', () => {
		const root = render(MobileNavDrawer, { open: true });
		expect(root.querySelector('[role="dialog"] a[href="/chat/new"]')).not.toBeNull();
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
		flushSync();
		expect(root.querySelector('[role="dialog"]')).toBeNull();

		page.url = new URL('http://localhost/preferences');
		const settings = render(MobileNavDrawer, { open: true });
		expect(settings.querySelector('[role="dialog"] a[href="/preferences"]')!.getAttribute('aria-current')).toBe('page');
	});
});
