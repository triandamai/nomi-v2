import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AddressRow from './AddressRow.svelte';
import SubdomainField from './SubdomainField.svelte';
import { tokenColor } from '../test-utils';

// Domain (Flow / Build — Workspace tools U16wU, Screen / Domain J2hPJ), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const labels = { field: 'Address', checking: 'Checking…', available: 'Available', suggestions: 'Free instead' };

describe('AddressRow', () => {
	it('is the pen’s 41px row: [12,14], outlined, the globe, the address in mono, its state on the right', async () => {
		await page.viewport(1512, 862);
		await render(AddressRow, { address: 'warung-kopi-menu.trian.space', href: 'https://warung-kopi-menu.trian.space', state: { label: 'Live', tone: 'live' } });
		const row = page.getByText('warung-kopi-menu.trian.space').element().closest('[data-part="address"]')!;
		expect([box(row).height, getComputedStyle(row).outlineColor]).toEqual([41, tokenColor('nm-border')]);
		expect(getComputedStyle(page.getByText('warung-kopi-menu.trian.space').element()).fontFamily).toMatch(/Mono/);
		await expect.element(page.getByRole('link', { name: /warung-kopi-menu/ })).toHaveAttribute('href', 'https://warung-kopi-menu.trian.space');
		await expect.element(page.getByText('Live')).toBeVisible();
	});

	it('says an address that’s waiting in the warning colour', async () => {
		await render(AddressRow, { address: 'kopi.trian.space', state: { label: 'on the next publish', tone: 'waiting' } });
		expect(getComputedStyle(page.getByText('on the next publish').element()).color).toBe(tokenColor('nm-warning'));
	});
});

describe('SubdomainField', () => {
	it('takes the name before the domain, and says it’s free', async () => {
		await render(SubdomainField, { value: 'warung-kopi', suffix: 'trian.space', check: { state: 'available' }, labels });
		await expect.element(page.getByRole('textbox', { name: 'Address' })).toHaveValue('warung-kopi');
		await expect.element(page.getByText('.trian.space')).toBeVisible();
		expect(getComputedStyle(page.getByText('Available').element()).color).toBe(tokenColor('nm-accent'));
	});

	it('says why a name can’t be had, and offers free ones', async () => {
		const onpick = vi.fn();
		await render(SubdomainField, { value: 'kopi', suffix: 'trian.space', check: { state: 'taken', message: 'That name is taken.', suggestions: ['kopi-1', 'kopi-bu-siti'] }, labels, onpick });
		expect(getComputedStyle(page.getByText('That name is taken.').element()).color).toBe(tokenColor('nm-danger'));
		await expect.element(page.getByRole('textbox', { name: 'Address' })).toHaveAttribute('aria-invalid', 'true');
		await page.getByRole('button', { name: 'kopi-bu-siti' }).click();
		expect(onpick).toHaveBeenCalledWith('kopi-bu-siti');
	});
});
