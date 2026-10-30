import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import NavigationSection from './NavigationSection.svelte';

describe('Navigation, identity & status section', () => {
	it('lays out every part from the design sheet', async () => {
		await render(NavigationSection);
		for (const caption of [
			'Nav Item / Default',
			'Nav Item / Active',
			'Nav Group Label',
			'Recent / Chat',
			'Recent / Project',
			'Avatar / User',
			'Mascot 20 / 32 / 96',
			'Agent Byline',
			'Crew Row',
			'Status / Live',
			'Status / Ready',
			'Status / Idle',
			'Status / Failed',
			'Status / Attention',
			'Status / Working',
			'Tag / No-go',
			'Tag / Go',
			'Tag / Neutral',
			'Section Label',
			'Meter',
		]) {
			expect(page.getByText(caption, { exact: true }).elements(), caption).toHaveLength(1);
		}
	});

	it('draws the mascot specimen at 20 and 32 only, as the pen does', async () => {
		const { container } = await render(NavigationSection);
		const caption = page.getByText('Mascot 20 / 32 / 96', { exact: true }).element();
		const sizes = caption.previousElementSibling!;
		expect([...sizes.querySelectorAll('svg')].map((mascot) => mascot.getAttribute('width'))).toEqual(['20', '32']);
		expect(getComputedStyle(sizes).columnGap).toBe('12px');
		expect(container.querySelectorAll('svg[width="96"]')).toHaveLength(0);
	});
});
