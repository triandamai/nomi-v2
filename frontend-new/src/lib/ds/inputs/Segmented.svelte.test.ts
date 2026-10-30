import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Segmented from './Segmented.svelte';
import { tokenBackground } from '../test-utils';

const options = [
	{ value: 'all', label: 'All', count: 4 },
	{ value: 'live', label: 'Live', count: 2 },
	{ value: 'building', label: 'Building', count: 1 },
];

describe('Segmented', () => {
	it('shows each choice with its count, the chosen one raised', async () => {
		await render(Segmented, { label: 'Projects', options, value: 'all' });
		const all = page.getByRole('radio', { name: 'All 4' });
		await expect.element(all).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByRole('radio', { name: 'Live 2' })).toHaveAttribute('aria-checked', 'false');
		expect(getComputedStyle(all.element()).backgroundColor).toBe(tokenBackground('nm-bg'));
		expect(getComputedStyle(all.element()).height).toBe('26px');
		const track = page.getByRole('radiogroup', { name: 'Projects' }).element();
		expect(getComputedStyle(track).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});

	it('changes choice by click and by arrow keys', async () => {
		let chosen = 'all';
		await render(Segmented, { label: 'Projects', options, value: 'all', onchange: (value: string) => (chosen = value) });
		await page.getByRole('radio', { name: 'Live 2' }).click();
		expect(chosen).toBe('live');
		await userEvent.keyboard('{ArrowRight}');
		expect(chosen).toBe('building');
		await expect.element(page.getByRole('radio', { name: 'Building 1' })).toHaveFocus();
		await userEvent.keyboard('{ArrowRight}');
		expect(chosen).toBe('all');
	});
});
