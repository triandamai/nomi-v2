// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/admin') } }));

// The collapse preference reads a media query on mount; jsdom has no matchMedia.
window.matchMedia ??= ((query: string) =>
	({ matches: false, media: query, addEventListener() {}, removeEventListener() {} }) as unknown as MediaQueryList) as typeof window.matchMedia;

import AdminSidebar from './AdminSidebar.svelte';

const FULL_ACCESS = {
	dashboard: 'manage',
	agents: 'manage',
	tools: 'manage',
	models: 'manage',
	integrations: 'manage',
	support: 'manage',
	user: 'manage',
	plans: 'manage',
	notifications: 'manage',
	maintenance: 'manage',
} as const;

let component: ReturnType<typeof mount> | null = null;

function render() {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(AdminSidebar, { target, props: { access: FULL_ACCESS, onappearance: () => {} } });
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

describe('AdminSidebar', () => {
	it('puts the nav links in their own scroll region so short screens can reach every page', () => {
		const target = render();
		const scroller = target.querySelector<HTMLElement>('[data-testid="admin-drawer-scroll"]');
		expect(scroller).not.toBeNull();
		const links = scroller!.querySelectorAll('a[href^="/admin"]');
		expect(links.length).toBeGreaterThanOrEqual(10);
	});

	it('keeps the footer actions outside the scroll region so they stay pinned', () => {
		const target = render();
		const scroller = target.querySelector<HTMLElement>('[data-testid="admin-drawer-scroll"]')!;
		const back = [...target.querySelectorAll('a')].find((a) => a.getAttribute('href') === '/');
		expect(back).toBeDefined();
		expect(scroller.contains(back!)).toBe(false);
	});
});
