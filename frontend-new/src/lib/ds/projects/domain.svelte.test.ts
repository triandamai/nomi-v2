import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AddressRow from './AddressRow.svelte';
import SubdomainField from './SubdomainField.svelte';
import DnsRecords from './DnsRecords.svelte';
import { tokenBackground as tokenBackgroundOf, tokenColor } from '../test-utils';

// Domain (Flow / Build — Workspace tools U16wU, Screen / Domain J2hPJ), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const dnsLabels = { heading: 'Set these at your registrar', type: 'Type', name: 'Name', value: 'Value', status: 'Status', found: 'found', waiting: 'waiting', copy: 'Copy value' };
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

describe('AddressRow, waiting', () => {
	it('draws the warning dot before what it waits for, as the pen does', async () => {
		await render(AddressRow, { address: 'warungkopibusiti.id', state: { label: 'waiting for DNS', tone: 'waiting' }, strong: true });
		const label = page.getByText('waiting for DNS').element();
		const dot = label.previousElementSibling!;
		expect([box(dot).width, getComputedStyle(dot).backgroundColor, getComputedStyle(label).color]).toEqual([6, tokenBackgroundOf('nm-warning'), tokenColor('nm-warning')]);
	});
});

describe('DnsRecords', () => {
	const records = [
		{ type: 'A', name: '@', value: '76.76.21.21', found: false },
		{ type: 'CNAME', name: 'www', value: 'warung-kopi-menu-3f9a12.trian.space', found: true },
		{ type: 'TXT', name: '_nomi-verify', value: 'nomi-verify=4be1c07', found: false },
	];

	it('is the pen’s table: its heading and when it was checked, a header on surface-2, a row per record', async () => {
		await page.viewport(1512, 862);
		await render(DnsRecords, { records, checked: 'last checked 08:51 · every 5 min', labels: dnsLabels });
		await expect.element(page.getByText('last checked 08:51 · every 5 min')).toBeVisible();
		const table = page.getByRole('table', { name: 'Set these at your registrar' }).element();
		const frame = table.closest('[data-part="records"]')!;
		expect([getComputedStyle(frame).outlineColor, getComputedStyle(frame).borderRadius]).toEqual([tokenColor('nm-border'), '6px']);
		const rows = table.querySelectorAll('tr');
		expect(rows).toHaveLength(4);
		expect(getComputedStyle(rows[0]).backgroundColor).toBe(tokenBackgroundOf('nm-surface-2'));
		expect([box(rows[1].querySelector('td span')!).width, box(rows[1]).height]).toEqual([56, 34]);
		expect(getComputedStyle(rows[1]).borderTopColor).toBe(tokenColor('nm-border'));
		expect(getComputedStyle(page.getByText('found', { exact: true }).element()).color).toBe(tokenColor('nm-accent'));
		expect(getComputedStyle(page.getByText('waiting', { exact: true }).first().element()).color).toBe(tokenColor('nm-warning'));
	});

	it('copies a record’s value', async () => {
		const write = vi.fn(async () => {});
		Object.defineProperty(navigator, 'clipboard', { value: { writeText: write }, configurable: true });
		await render(DnsRecords, { records, checked: '', labels: dnsLabels });
		await page.getByRole('button', { name: 'Copy value nomi-verify=4be1c07' }).click();
		expect(write).toHaveBeenCalledWith('nomi-verify=4be1c07');
	});
});
