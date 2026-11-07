// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';

const nav = vi.hoisted(() => ({ navigating: { to: null as { url: URL } | null } }));
vi.mock('$app/state', () => nav);

const { default: NavProgress } = await import('./NavProgress.svelte');

let instance: Record<string, unknown> | undefined;
beforeEach(() => vi.useFakeTimers());
afterEach(() => {
	if (instance) unmount(instance);
	instance = undefined;
	nav.navigating.to = null;
	vi.useRealTimers();
	document.body.innerHTML = '';
});

describe('NavProgress', () => {
	it('shows a bar once a page has been loading for a beat', () => {
		nav.navigating.to = { url: new URL('http://localhost/projects') };
		const target = document.body.appendChild(document.createElement('div'));
		instance = mount(NavProgress, { target });
		flushSync();
		expect(target.querySelector('[role="progressbar"]')).toBeNull();
		vi.advanceTimersByTime(200);
		flushSync();
		expect(target.querySelector('[role="progressbar"]')).not.toBeNull();
	});

	it('shows nothing while no page is loading', () => {
		const target = document.body.appendChild(document.createElement('div'));
		instance = mount(NavProgress, { target });
		vi.advanceTimersByTime(1000);
		flushSync();
		expect(target.querySelector('[role="progressbar"]')).toBeNull();
	});
});
