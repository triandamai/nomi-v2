import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Inbox } from '@lucide/svelte';
import SettingRow from './SettingRow.svelte';
import SettingNote from './SettingNote.svelte';
import PageToc from './PageToc.svelte';
import RelatedLinks from './RelatedLinks.svelte';
import SettingsTip from './SettingsTip.svelte';
import AutosaveNote from './AutosaveNote.svelte';
import SettingsFixture from './SettingsFixture.svelte';
import Select from '../inputs/Select.svelte';
import { tokenColor } from '../test-utils';

// Settings › General's pieces (App / Settings / General m7lvY1), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('SettingsGroup and SettingRow', () => {
	it('puts a group’s name over its rows, each 57 tall with its control at the end', async () => {
		await render(SettingsFixture);
		await expect.element(page.getByRole('heading', { level: 2, name: 'Appearance' })).toBeVisible();
		const row = page.getByText('Reduce motion').element().closest('.border-b')!;
		expect(box(row).height).toBe(57);
		expect(box(page.getByRole('switch').element()).right).toBe(box(row).right);
	});

	it('writes a machine’s value in mono, and an action that can’t be undone in the danger colour', async () => {
		await render(SettingRow, { title: 'Delete account', description: 'ko••••@gmail.com', mono: true, danger: true });
		expect(getComputedStyle(page.getByText('Delete account').element()).color).toBe(tokenColor('nm-danger'));
		expect(getComputedStyle(page.getByText('ko••••@gmail.com').element()).fontFamily).toMatch(/Geist Mono/);
	});

	it('adds a quiet note under the rows', async () => {
		await render(SettingNote, { text: 'Signing in with Google doesn’t give the crew your Gmail.' });
		expect(box(page.getByText(/Signing in with Google/).element().parentElement!).height).toBe(40);
	});
});

describe('the page column', () => {
	it('lists the page’s groups, the one in view marked with a 2px stroke', async () => {
		await render(PageToc, { label: 'On this page', links: [{ href: '#account', label: 'Account & sign-in' }, { href: '#language', label: 'Language & region' }], active: '#account' });
		const current = page.getByRole('link', { name: 'Account & sign-in' }).element();
		expect([getComputedStyle(current).borderLeftWidth, box(current).height]).toEqual(['2px', 27]);
		await expect.element(page.getByRole('link', { name: 'Account & sign-in' })).toHaveAttribute('aria-current', 'location');
		expect(getComputedStyle(page.getByRole('link', { name: 'Language & region' }).element()).borderLeftWidth).toBe('1px');
	});

	it('links to related places, 28 tall', async () => {
		await render(RelatedLinks, { label: 'Related', links: [{ href: '/inbox', label: 'Inbox', icon: Inbox }] });
		expect(box(page.getByRole('link', { name: 'Inbox' }).element()).height).toBe(28);
	});

	it('says how the page saves, with the accent check', async () => {
		await render(SettingsTip, { text: 'Changes save as you go.' });
		await render(AutosaveNote, { text: 'changes save automatically' });
		expect(getComputedStyle(page.getByText('changes save automatically').element()).fontFamily).toMatch(/Geist Mono/);
	});
});

describe('Select', () => {
	it('is the platform’s select, 30 tall, picking with the keyboard', async () => {
		const onchange = vi.fn();
		await render(Select, { label: 'Time zone', options: [{ value: 'Asia/Jakarta', label: 'Asia/Jakarta · WIB' }, { value: 'Asia/Makassar', label: 'Asia/Makassar · WITA' }], value: 'Asia/Jakarta', onchange });
		const select = page.getByRole('combobox', { name: 'Time zone' });
		expect(box(select.element()).height).toBe(30);
		await userEvent.selectOptions(select.element(), 'Asia/Makassar');
		expect(onchange).toHaveBeenCalledWith('Asia/Makassar');
	});

	it('shows a placeholder while nothing is chosen', async () => {
		await render(Select, { label: 'Time zone', options: [{ value: 'UTC', label: 'UTC' }], value: '', placeholder: 'Choose your time zone' });
		await expect.element(page.getByRole('combobox', { name: 'Time zone' })).toHaveValue('');
		await expect.element(page.getByRole('option', { name: 'Choose your time zone' })).toBeDisabled();
	});
});
