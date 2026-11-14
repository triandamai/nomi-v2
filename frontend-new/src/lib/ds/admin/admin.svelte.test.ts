import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { House, Users } from '@lucide/svelte';
import AccessEditor from './AccessEditor.svelte';
import AdminNav from './AdminNav.svelte';
import AdminTableFixture from './AdminTableFixture.svelte';
import ConfirmDialog from './ConfirmDialog.svelte';
import FilterBar from './FilterBar.svelte';
import FormSectionFixture from './FormSectionFixture.svelte';
import LevelBadge from './LevelBadge.svelte';
import Notice from './Notice.svelte';
import PageHeader from './PageHeader.svelte';
import Pager from './Pager.svelte';
import SecretField from './SecretField.svelte';
import SecretReveal from './SecretReveal.svelte';
import Select from './Select.svelte';
import StatTile from './StatTile.svelte';
import TextArea from './TextArea.svelte';
import UsageChart from './UsageChart.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

const mono = (element: Element) => getComputedStyle(element).fontFamily.includes('Geist Mono');

describe('AdminNav', () => {
	it('names the console, leads back to Nomi, and lists only the areas given, the open one current', async () => {
		await render(AdminNav, {
			label: 'Admin',
			mark: 'admin',
			homeHref: '/admin',
			backHref: '/',
			backLabel: 'Back to Nomi',
			active: '/admin/users',
			groups: [
				{ label: 'Dashboard', items: [{ href: '/admin', icon: House, label: 'Overview' }] },
				{ label: 'People', items: [{ href: '/admin/users', icon: Users, label: 'Users & access', count: 12 }] },
			],
		});
		const nav = page.getByRole('navigation', { name: 'Admin' });
		await expect.element(nav.getByRole('link', { name: 'Back to Nomi' })).toHaveAttribute('href', '/');
		await expect.element(nav.getByRole('link', { name: /Users & access/ })).toHaveAttribute('aria-current', 'page');
		await expect.element(nav.getByRole('link', { name: 'Overview' })).not.toHaveAttribute('aria-current');
		expect(getComputedStyle(page.getByText('admin', { exact: true }).element()).textTransform).toBe('uppercase');
		expect(nav.element().getBoundingClientRect().width).toBe(248);
	});
});

describe('PageHeader', () => {
	it('titles the page, says what it is for, and leads back from a page inside another', async () => {
		// `intro` is also a Svelte option, so the props go under `props`.
		await render(PageHeader, { props: { title: 'Plans', intro: 'The plans people can be on.', back: { href: '/admin/plans', label: 'Plans' }, meta: 'pro' } });
		await expect.element(page.getByRole('heading', { level: 1, name: 'Plans' })).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'Plans' })).toHaveAttribute('href', '/admin/plans');
		expect(getComputedStyle(page.getByRole('heading').element()).fontSize).toBe('28px');
		expect(mono(page.getByText('pro', { exact: true }).element())).toBe(true);
	});
});

describe('StatTile', () => {
	it('shows the real figure in mono, the exact one in its title, and the live dot only while live', async () => {
		const { rerender } = await render(StatTile, { label: 'Tokens today', value: '1.2K', exact: '1,234', note: 'across every agent' });
		const value = page.getByText('1.2K').element();
		expect(mono(value)).toBe(true);
		expect(value.getAttribute('title')).toBe('1,234');
		expect(document.querySelector('[data-dot]')).toBeNull();
		await rerender({ label: 'Running now', value: '2', live: true });
		expect(getComputedStyle(document.querySelector('[data-dot]')!).backgroundColor).toBe(tokenBackground('nm-accent'));
	});
});

describe('FormSection', () => {
	it('is a named region with its fields and a footer for its actions, outlined in danger when it can’t be undone', async () => {
		const { rerender } = await render(FormSectionFixture, {});
		const region = page.getByRole('region', { name: 'App domains' });
		await expect.element(region.getByRole('textbox', { name: 'Apps base domain' })).toBeVisible();
		await expect.element(region.getByRole('button', { name: 'Save' })).toBeVisible();
		expect(getComputedStyle(region.element()).boxShadow).toBe('none');
		await rerender({ danger: true });
		expect(getComputedStyle(page.getByRole('region').element()).outlineColor).toBe(tokenColor('nm-danger-border'));
	});
});

describe('LevelBadge', () => {
	it('names the level in mono capitals, a step stronger each level', async () => {
		await render(LevelBadge, { level: 'manage', label: 'Manage' });
		const badge = page.getByText('Manage').element();
		expect(mono(badge)).toBe(true);
		expect(getComputedStyle(badge).textTransform).toBe('uppercase');
		expect(getComputedStyle(badge).color).toBe(tokenColor('nm-text'));
	});
});

describe('Notice', () => {
	it('says a failure as an alert, with the backend’s reason in mono', async () => {
		await render(Notice, { kind: 'error', text: 'Couldn’t save the plan.', detail: 'another plan already uses that slug' });
		await expect.element(page.getByRole('alert')).toHaveTextContent(/Couldn’t save the plan\./);
		expect(mono(page.getByText('another plan already uses that slug').element())).toBe(true);
	});

	it('says good news as a status', async () => {
		await render(Notice, { kind: 'success', text: 'Saved.' });
		await expect.element(page.getByRole('status')).toHaveTextContent('Saved.');
	});
});

describe('ConfirmDialog', () => {
	it('asks before something that can’t be undone, and cancels on Escape', async () => {
		const onconfirm = vi.fn();
		await render(ConfirmDialog, { open: true, title: 'Delete Pro?', body: 'People can’t be put on it again.', confirmLabel: 'Delete plan', cancelLabel: 'Cancel', onconfirm });
		const dialog = page.getByRole('dialog', { name: 'Delete Pro?' });
		await expect.element(dialog).toHaveAccessibleDescription('People can’t be put on it again.');
		await dialog.getByRole('button', { name: 'Delete plan' }).click();
		expect(onconfirm).toHaveBeenCalledOnce();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});

	it('can’t be confirmed twice while the change is on its way', async () => {
		await render(ConfirmDialog, { open: true, title: 'Revoke key?', body: 'Products using it stop getting answers.', confirmLabel: 'Revoke', cancelLabel: 'Cancel', busy: true, onconfirm: () => {} });
		await expect.element(page.getByRole('button', { name: 'Revoke' })).toBeDisabled();
	});
});

describe('TextArea', () => {
	it('is a labelled multi-line field that counts what’s left of its limit', async () => {
		await render(TextArea, { label: 'Message', value: 'Hello', maxlength: 500, name: 'body' });
		const field = page.getByRole('textbox', { name: 'Message' });
		await expect.element(field).toHaveValue('Hello');
		await expect.element(page.getByText('5/500')).toBeVisible();
		await field.fill('Hello there');
		await expect.element(page.getByText('11/500')).toBeVisible();
	});

	it('says what’s wrong in place of the helper', async () => {
		await render(TextArea, { label: 'Message', helper: 'Shown in the inbox.', error: 'Add a message.' });
		await expect.element(page.getByRole('textbox', { name: 'Message' })).toHaveAccessibleDescription('Add a message.');
		await expect.element(page.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
	});
});

describe('Select', () => {
	it('is the platform’s own select, with its label and helper', async () => {
		const onchange = vi.fn();
		await render(Select, { label: 'Plan', helper: 'Hidden plans too.', options: [{ value: 'free', label: 'Free' }, { value: 'pro', label: 'Pro' }], value: 'free', onchange });
		const select = page.getByRole('combobox', { name: 'Plan' });
		await expect.element(select).toHaveAccessibleDescription('Hidden plans too.');
		await select.selectOptions('pro');
		expect(onchange).toHaveBeenCalledWith('pro');
	});
});

describe('SecretField', () => {
	it('never shows a stored secret: only that one is set, and offers to forget it', async () => {
		await render(SecretField, { label: 'API key', name: 'api_key', set: true, status: 'Saved here. Leave empty to keep it.', forget: { name: 'clear_api_key', label: 'Forget the saved key' } });
		const field = page.getByLabelText('API key');
		await expect.element(field).toHaveValue('');
		await expect.element(field).toHaveAttribute('type', 'password');
		await expect.element(field).toHaveAttribute('autocomplete', 'new-password');
		await expect.element(field).toHaveAccessibleDescription('Saved here. Leave empty to keep it.');
		await expect.element(page.getByRole('checkbox', { name: 'Forget the saved key' })).toBeVisible();
	});

	it('has nothing to forget when none is set', async () => {
		await render(SecretField, { label: 'API key', name: 'api_key', set: false, status: 'Not set.', forget: { name: 'clear', label: 'Forget the saved key' } });
		await expect.element(page.getByRole('checkbox')).not.toBeInTheDocument();
	});
});

describe('SecretReveal', () => {
	it('shows a secret once, in mono, to copy, and goes when the person has it', async () => {
		const ondone = vi.fn();
		await render(SecretReveal, { title: 'Copy this key now: it won’t be shown again.', secret: 'nsk_0123456789abcdef', copyLabel: 'Copy', copiedLabel: 'Copied', doneLabel: 'Done', ondone });
		expect(mono(page.getByText('nsk_0123456789abcdef').element())).toBe(true);
		await page.getByRole('button', { name: 'Done' }).click();
		expect(ondone).toHaveBeenCalledOnce();
	});
});

describe('FilterBar', () => {
	it('searches by the address, keeping what else the list was showing', async () => {
		await render(FilterBar, { label: 'Search people', placeholder: 'Search by email', query: 'ana', submitLabel: 'Search', keep: { size: '50' } });
		const form = page.getByRole('search', { name: 'Search people' }).element() as HTMLFormElement;
		expect(form.method).toBe('get');
		expect(Object.fromEntries(new FormData(form))).toEqual({ size: '50', query: 'ana' });
		await page.getByRole('searchbox', { name: 'Search people' }).fill('budi@kopikita.id');
		expect(Object.fromEntries(new FormData(form))).toEqual({ size: '50', query: 'budi@kopikita.id' });
	});
});

describe('Pager', () => {
	it('says where the person is and links the pages either side, but not past either end', async () => {
		await render(Pager, { label: 'People pages', summary: '1–20 of 134', previousLabel: 'Previous', nextLabel: 'Next', nextHref: '?page=2' });
		const nav = page.getByRole('navigation', { name: 'People pages' });
		expect(mono(page.getByText('1–20 of 134').element())).toBe(true);
		await expect.element(nav.getByRole('link', { name: 'Next' })).toHaveAttribute('href', '?page=2');
		await expect.element(nav.getByRole('button', { name: 'Previous' })).toBeDisabled();
	});
});

describe('AdminTable', () => {
	const rows = [
		{ slug: 'pro', name: 'Pro', tokens: 5_000_000 },
		{ slug: 'free', name: 'Free', tokens: 1_000_000 },
		{ slug: 'team', name: 'Team', tokens: 20_000_000, hidden: true },
	];
	const names = () => [...document.querySelectorAll('tbody tr')].map((row) => row.querySelector('td')!.textContent!.trim());

	it('is a table named for its rows, with mono section labels for its columns', async () => {
		await page.viewport(1280, 800);
		await render(AdminTableFixture, { rows });
		await expect.element(page.getByRole('table', { name: 'Plans' })).toBeVisible();
		const header = page.getByRole('columnheader', { name: 'Id' }).element();
		expect(mono(header)).toBe(true);
		expect(getComputedStyle(header).textTransform).toBe('uppercase');
		expect(names()).toEqual(['Pro', 'Free', 'Team']);
		expect(getComputedStyle(document.querySelectorAll('tbody tr')[2]).opacity).toBe('0.6');
	});

	it('leaves out on a phone the columns that don’t fit there', async () => {
		await page.viewport(390, 800);
		await render(AdminTableFixture, { rows });
		await expect.element(page.getByRole('columnheader', { name: /Plan/ })).toBeVisible();
		await expect.element(page.getByText('pro', { exact: true })).not.toBeVisible();
	});

	it('sorts by a column on a click of its name, then the other way', async () => {
		await render(AdminTableFixture, { rows });
		const tokens = page.getByRole('columnheader', { name: /Tokens a month/ });
		await tokens.getByRole('button').click();
		await expect.element(tokens).toHaveAttribute('aria-sort', 'ascending');
		expect(names()).toEqual(['Free', 'Pro', 'Team']);
		await tokens.getByRole('button').click();
		await expect.element(tokens).toHaveAttribute('aria-sort', 'descending');
		expect(names()).toEqual(['Team', 'Pro', 'Free']);
		await expect.element(page.getByRole('columnheader', { name: /Plan/ })).toHaveAttribute('aria-sort', 'none');
		expect(page.getByText('Id', { exact: true }).element().closest('th')!.hasAttribute('aria-sort')).toBe(false);
	});
});

describe('AccessEditor', () => {
	const areas = [
		{ key: 'plans', label: 'Plans', description: 'The plans people can be on.' },
		{ key: 'support', label: 'Support', description: 'Articles, keys and questions.' },
		{ key: 'models', label: 'Models', description: 'Providers and models.' },
	];
	const levelLabels = { none: 'None', view: 'View', write: 'Write', manage: 'Manage' };

	it('changes an area’s level, never above the editor’s own, and sends the role with the form', async () => {
		const onchange = vi.fn();
		await render(AccessEditor, { label: 'Admin access', areas, value: { plans: 'view' }, max: { plans: 'write', support: 'manage', models: null }, levelLabels, lockedNote: 'Above your own access.', name: 'access', onchange });
		const plans = page.getByRole('radiogroup', { name: 'Plans' });
		await expect.element(plans.getByRole('radio', { name: 'View' })).toHaveAttribute('aria-checked', 'true');
		await expect.element(plans.getByRole('radio', { name: 'Manage' })).toBeDisabled();
		await plans.getByRole('radio', { name: 'Write' }).click();
		expect(onchange).toHaveBeenLastCalledWith({ plans: 'write' });
		await userEvent.keyboard('{ArrowLeft}');
		expect(onchange).toHaveBeenLastCalledWith({ plans: 'view' });
		await expect.element(plans.getByRole('radio', { name: 'View' })).toHaveFocus();
		expect(JSON.parse((document.querySelector('input[name="access"]') as HTMLInputElement).value)).toEqual({ plans: 'view' });
		// An area the editor doesn't hold can't be given at all.
		await expect.element(page.getByRole('radiogroup', { name: 'Models' }).getByRole('radio', { name: 'View' })).toBeDisabled();
	});

	it('leaves alone an area the person holds above the editor’s own, and says why', async () => {
		await render(AccessEditor, { label: 'Admin access', areas, value: { support: 'manage' }, max: { plans: 'manage', support: 'write' }, levelLabels, lockedNote: 'Above your own access.' });
		const support = page.getByRole('radiogroup', { name: 'Support' });
		await expect.element(support).toHaveAttribute('aria-disabled', 'true');
		await expect.element(support.getByRole('radio', { name: 'None' })).toBeDisabled();
		await expect.element(page.getByText('Above your own access.')).toBeVisible();
	});
});

describe('UsageChart', () => {
	it('draws each day from zero against an honest scale, and says its figures on focus', async () => {
		await render(UsageChart, {
			title: 'Chat tokens per day',
			max: 2000,
			axis: ['2K', '1K', '0'],
			bars: [
				{ key: '2026-11-01', value: 1000, tick: '1', tooltip: ['Sun 1 Nov', '1,000 chat tokens'] },
				{ key: '2026-11-02', value: 0, tooltip: ['Mon 2 Nov', '0 chat tokens'] },
				{ key: '2026-11-03', value: 2000, tooltip: ['Tue 3 Nov', '2,000 chat tokens'] },
			],
		});
		await expect.element(page.getByRole('figure', { name: 'Chat tokens per day' })).toBeVisible();
		const bars = [...document.querySelectorAll<HTMLElement>('[data-bar]')];
		const track = bars[0].parentElement!.getBoundingClientRect().height;
		expect(bars.map((bar) => Math.round((bar.getBoundingClientRect().height / track) * 100))).toEqual([50, 0, 100]);
		expect(getComputedStyle(bars[0]).backgroundImage).toBe('none');
		await page.getByRole('button', { name: 'Sun 1 Nov, 1,000 chat tokens' }).click();
		await expect.element(page.getByRole('tooltip')).toHaveTextContent(/Sun 1 Nov\s*1,000 chat tokens/);
		await userEvent.keyboard('{ArrowRight}');
		await expect.element(page.getByRole('button', { name: 'Mon 2 Nov, 0 chat tokens' })).toHaveFocus();
	});
});
