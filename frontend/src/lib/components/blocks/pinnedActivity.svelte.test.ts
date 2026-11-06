// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import PinnedActivity from './PinnedActivity.svelte';

// The card pinned above the composer: one action per state, steps folded under it.

afterEach(() => {
	document.body.innerHTML = '';
});

function render(props: Record<string, unknown>) {
	const target = document.body.appendChild(document.createElement('div'));
	const component = mount(PinnedActivity, { target, props: { steps: [], ...props } as never });
	flushSync();
	return { target, done: () => unmount(component) };
}

describe('PinnedActivity', () => {
	it('while working: says what it is doing, the time so far, and stops on click', () => {
		const onstop = vi.fn();
		const { target, done } = render({ status: 'working', headline: 'Mira is searching the web…', elapsed: '00:42', onstop, steps: [{ label: 'Searched the web', state: 'done' }, { label: 'Running tests', state: 'running' }] });
		expect(target.querySelector('[role="status"]')?.textContent).toBe('Mira is searching the web…');
		expect(target.textContent).toContain('00:42');
		expect(target.textContent).toContain('2');
		target.querySelector<HTMLButtonElement>('button[aria-keyshortcuts="Escape"]')?.click();
		expect(onstop).toHaveBeenCalledOnce();
		done();
	});

	it('waiting: offers to go to the question, with no stop and no dismiss', () => {
		const onjump = vi.fn();
		const { target, done } = render({ status: 'waiting', headline: 'Mira needs your answer', jumpLabel: 'Go to it', onjump });
		expect(target.querySelector('button[aria-keyshortcuts="Escape"]')).toBeNull();
		const go = [...target.querySelectorAll('button')].find((button) => button.textContent?.includes('Go to it'));
		go?.click();
		expect(onjump).toHaveBeenCalledOnce();
		expect(target.querySelectorAll('button').length).toBe(1);
		done();
	});

	it('done: can be dismissed', () => {
		const ondismiss = vi.fn();
		const { target, done } = render({ status: 'done', headline: 'Mira is done', elapsed: '01:12', ondismiss });
		expect(target.querySelector('section')?.dataset.status).toBe('done');
		target.querySelector<HTMLButtonElement>('button')?.click();
		expect(ondismiss).toHaveBeenCalledOnce();
		done();
	});
});
