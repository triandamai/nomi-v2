// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import SplitButton from './SplitButton.svelte';

let instance: Record<string, unknown> | undefined;
afterEach(() => {
	if (instance) unmount(instance);
	instance = undefined;
	document.body.innerHTML = '';
});

describe('SplitButton', () => {
	it('does its main thing on the label and opens the grouped menu from the chevron', async () => {
		const main = vi.fn();
		const picked = vi.fn();
		const root = document.body.appendChild(document.createElement('div'));
		instance = mount(SplitButton, {
			target: root,
			props: {
				label: 'Share',
				moreLabel: 'More tools',
				onclick: main,
				items: [{ group: 'Project' }, { label: 'Settings', onselect: picked }, 'divider', { label: 'Export' }],
			},
		});
		root.querySelector<HTMLButtonElement>('.split__main')!.click();
		expect(main).toHaveBeenCalledOnce();
		expect(root.querySelector('[role="menu"]')).toBeNull();

		root.querySelector<HTMLButtonElement>('[aria-label="More tools"]')!.click();
		flushSync();
		await tick();
		expect(root.querySelector('[role="group"]')?.textContent).toContain('Project');
		root.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')[0].click();
		flushSync();
		expect(picked).toHaveBeenCalledOnce();
		expect(main).toHaveBeenCalledOnce();
	});
});
