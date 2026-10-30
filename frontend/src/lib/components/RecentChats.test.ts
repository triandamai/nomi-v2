// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import RecentChats from './RecentChats.svelte';
import type { RecentsState } from '$lib/drawerRecents';

const chats = [
	{ id: 'a', title: 'Trip plan', href: '/chat/a', pinned: false },
	{ id: 'b', title: 'Budget', href: '/chat/b', pinned: true },
];

let component: ReturnType<typeof mount> | null = null;

function render(props: { state: RecentsState; expanded?: boolean; activeId?: string | null; ontoggle?: () => void; onnavigate?: () => void; onretry?: () => void }) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(RecentChats, { target, props: { expanded: true, ontoggle: () => {}, ...props } });
	flushSync();
	return target;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

describe('RecentChats', () => {
	it('lists each recent chat as a link to it, marking the open one', () => {
		const target = render({ state: { status: 'ready', chats }, activeId: 'b' });
		const links = [...target.querySelectorAll<HTMLAnchorElement>('nav a')].filter((a) => a.getAttribute('href')?.startsWith('/chat/'));
		expect(links.map((a) => [a.textContent?.trim(), a.getAttribute('href')])).toEqual([
			['Trip plan', '/chat/a'],
			['Budget', '/chat/b'],
		]);
		expect(links[0].getAttribute('aria-current')).toBeNull();
		expect(links[1].getAttribute('aria-current')).toBe('page');
	});

	it('marks a pinned chat as pinned', () => {
		const target = render({ state: { status: 'ready', chats } });
		expect(target.querySelector('a[href="/chat/a"] [role="img"]')).toBeNull();
		expect(target.querySelector('a[href="/chat/b"] [role="img"]')?.getAttribute('aria-label')).toBe('Pinned');
	});

	it('closes the drawer when a chat is picked', () => {
		const onnavigate = vi.fn();
		const target = render({ state: { status: 'ready', chats }, onnavigate });
		target.querySelector<HTMLAnchorElement>('a[href="/chat/a"]')!.addEventListener('click', (e) => e.preventDefault());
		target.querySelector<HTMLAnchorElement>('a[href="/chat/a"]')!.click();
		expect(onnavigate).toHaveBeenCalledOnce();
	});

	it('links to every chat from the end of the list', () => {
		const target = render({ state: { status: 'ready', chats } });
		expect(target.querySelector('a[href="/chats"]')?.textContent?.trim()).toBe('All chats');
	});

	it('says so when there are no recent chats', () => {
		const target = render({ state: { status: 'ready', chats: [] } });
		expect(target.textContent).toContain('No recent chats');
		expect(target.querySelector('a[href^="/chat/"]')).toBeNull();
	});

	it('shows that it is loading', () => {
		const target = render({ state: { status: 'loading' } });
		expect(target.querySelector('[role="status"]')?.textContent).toContain('Loading recent chats');
		expect(target.querySelector('[aria-busy="true"]')).not.toBeNull();
	});

	it('offers a retry when the chats could not be loaded', () => {
		const onretry = vi.fn();
		const target = render({ state: { status: 'error' }, onretry });
		expect(target.textContent).toContain("Couldn't load recent chats.");
		[...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Try again')!.click();
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('collapses under its heading', () => {
		const ontoggle = vi.fn();
		const target = render({ state: { status: 'ready', chats }, expanded: false, ontoggle });
		const heading = target.querySelector<HTMLButtonElement>('button[aria-expanded]')!;
		expect(heading.textContent?.trim()).toBe('Recents');
		expect(heading.getAttribute('aria-expanded')).toBe('false');
		expect(target.querySelector('a[href="/chat/a"]')).toBeNull();
		heading.click();
		expect(ontoggle).toHaveBeenCalledOnce();
	});
});
