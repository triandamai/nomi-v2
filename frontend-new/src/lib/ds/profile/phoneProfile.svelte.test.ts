import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import PhoneProfileHeader from './PhoneProfileHeader.svelte';
import PhoneActivity from './PhoneActivity.svelte';
import { heatmap } from '$lib/settings/profile';

// Flow / Mobile — You & Settings, step 7 (Profile): the person at the top, and a quarter of activity.

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PhoneProfileHeader', () => {
	it('names the page, rounds the picture at 72 and changes the photo from a 44 target', async () => {
		await page.viewport(390, 844);
		const onphoto = vi.fn();
		await render(PhoneProfileHeader, { name: 'Siti Rahayu', line: '@siti · joined 3 Oct 2026', photo: 'Change photo', onphoto });
		await expect.element(page.getByRole('heading', { level: 1, name: 'Siti Rahayu' })).toBeVisible();
		await expect.element(page.getByText('@siti · joined 3 Oct 2026')).toBeVisible();
		const initials = page.getByText('SR').element().parentElement!;
		expect(Math.round(box(initials).width)).toBe(72);
		expect(getComputedStyle(initials).borderRadius).not.toBe('0px');
		const change = page.getByRole('button', { name: 'Change photo' });
		expect(Math.round(box(change.element()).height)).toBe(44);
		await change.click();
		expect(onphoto).toHaveBeenCalledOnce();
	});

	it('shows the photo once there is one', async () => {
		await render(PhoneProfileHeader, { name: 'Siti', line: 'joined 3 Oct 2026', src: 'data:image/gif;base64,R0lGODlhAQABAAAAACw=', photo: 'Change photo', onphoto: () => {} });
		expect(document.querySelector('img')).not.toBeNull();
		await expect.element(page.getByText('S', { exact: true })).not.toBeInTheDocument();
	});
});

describe('PhoneActivity', () => {
	it('fills the width with 13 weeks, under the two numbers', async () => {
		await page.viewport(390, 844);
		const map = heatmap([{ date: '2026-10-20', tokens: 5000 }], '2026-10-21', 'en', 13);
		await render(PhoneActivity, {
			tiles: [
				{ label: 'Lifetime tokens', value: '1.2M', note: 'since 3 Oct 2026' },
				{ label: 'Current streak', value: '4 days', note: 'best is 9 · 5 to go' },
			],
			total: '5K tokens in the last 13 weeks',
			weeks: map.weeks,
			note: 'Most active Tuesday',
		});
		await expect.element(page.getByText('1.2M')).toBeVisible();
		await expect.element(page.getByText('Current streak')).toBeVisible();
		const grid = page.getByRole('img', { name: '5K tokens in the last 13 weeks' }).element();
		expect(grid.children).toHaveLength(13);
		const cell = grid.querySelector('span[title]')!;
		expect(Math.round(box(cell).height)).toBe(18);
		expect(box(cell).width).toBeGreaterThan(18);
		await expect.element(page.getByText('Most active Tuesday')).toBeVisible();
	});
});
