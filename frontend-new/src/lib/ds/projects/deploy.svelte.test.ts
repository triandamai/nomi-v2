import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import DeployRow from './DeployRow.svelte';
import { tokenColor } from '../test-utils';

// Deployments (Flow / Build — Workspace tools U16wU, Screen / Deployments S6Xq9H), measured
// against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const labels = { live: 'Live', failed: 'Failed', running: 'Publishing', rollback: 'Roll back to {number}' };

describe('DeployRow', () => {
	it('is 62 high over its line: the number in mono (40), what it is and when, Live on the right', async () => {
		await page.viewport(1512, 862);
		await render(DeployRow, { number: '#2', title: 'Deploy #2', meta: '08:42 today · by Mario', state: 'live', labels });
		const row = page.getByText('Deploy #2').element().closest('li')!;
		expect([box(row).height, getComputedStyle(row).borderBottomColor]).toEqual([62, tokenColor('nm-border')]);
		const number = page.getByText('#2', { exact: true }).element();
		expect([box(number).width, getComputedStyle(number).fontFamily]).toEqual([40, expect.stringMatching(/Mono/)]);
		await expect.element(page.getByText('Live')).toBeVisible();
	});

	it('offers to roll back to an earlier deploy', async () => {
		const onrollback = vi.fn();
		await render(DeployRow, { number: '#1', title: 'First deploy', meta: '08:18 today', state: 'rollback', labels, onrollback });
		await page.getByRole('button', { name: 'Roll back to #1' }).click();
		expect(onrollback).toHaveBeenCalledOnce();
	});

	it('says a failed deploy failed, and why, in the danger colour', async () => {
		await render(DeployRow, { number: '#3', title: 'Deploy #3', meta: '08:50 today', state: 'failed', error: 'Missing src/lib/menu.ts', labels });
		expect(getComputedStyle(page.getByText('Failed').element()).color).toBe(tokenColor('nm-danger'));
		await expect.element(page.getByText('Missing src/lib/menu.ts')).toBeVisible();
		expect(page.getByRole('button').elements()).toHaveLength(0);
	});
});
