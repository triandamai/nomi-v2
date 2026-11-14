import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SecretRow from './SecretRow.svelte';
import { tokenColor } from '../test-utils';

// Environment (Flow / Build — Workspace tools U16wU, Screen / Environment r85wCf).

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

describe('SecretRow', () => {
	it('names the variable in mono with when it changed, and never shows its value', async () => {
		const onremove = vi.fn();
		await render(SecretRow, { name: 'ADMIN_PIN', meta: 'updated 2d ago · Mario', masked: '••••••', remove: 'Remove ADMIN_PIN', onremove });
		const name = page.getByText('ADMIN_PIN', { exact: true }).element();
		expect(getComputedStyle(name).fontFamily).toMatch(/Mono/);
		expect(getComputedStyle(page.getByText('••••••').element()).color).toBe(tokenColor('nm-text-2'));
		const row = name.closest('li')!;
		expect(getComputedStyle(row).borderBottomColor).toBe(tokenColor('nm-border'));
		await page.getByRole('button', { name: 'Remove ADMIN_PIN' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});

	it('offers nothing to someone who can’t change it', async () => {
		await render(SecretRow, { name: 'DATABASE_URL', meta: 'managed', masked: 'set by Nomi', remove: 'Remove DATABASE_URL' });
		expect(page.getByRole('button').elements()).toHaveLength(0);
	});
});
