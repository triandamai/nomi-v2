import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet } from 'svelte';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ProviderCard from './ProviderCard.svelte';
import ConnectRow from './ConnectRow.svelte';
import WhoPays from './WhoPays.svelte';
import ConnectFixture from './ConnectFixture.svelte';
import Select from '../inputs/Select.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Models & providers' pieces (App / Settings / Models & providers RbTgo, Connect voUxB).

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const models = [
	{ key: 'admin:a1', label: 'deepseek-flash', enabled: true, isDefault: true, month: '13.5K' },
	{ key: 'admin:a2', label: 'gemini-pro', enabled: false, isDefault: false, month: '—' },
];
const columns = { model: 'Model', month: 'This month', picker: 'In picker' };

describe('ProviderCard', () => {
	it('opens to its models, 61 tall closed, the header on the surface when open', async () => {
		await page.viewport(1512, 862);
		await render(ProviderCard, { name: 'Nomi', mascot: 'nomi', detail: 'Free plan · 21.8K / 1M tokens this month', count: '1 of 2 in picker', models, columns, defaultLabel: 'Default for chats', empty: 'None' });
		const toggle = page.getByRole('button', { name: /Nomi/ });
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'false');
		expect(box(document.querySelector('section')!).height).toBe(61);
		await toggle.click();
		await expect.element(page.getByRole('table')).toBeVisible();
		expect(getComputedStyle(document.querySelector('header')!).backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(box(page.getByRole('row').nth(1).element()).height).toBe(38);
		await expect.element(page.getByText('Default for chats')).toBeVisible();
	});

	it('switches a model in or out of the picker', async () => {
		const onshow = vi.fn();
		await render(ProviderCard, { name: 'Nomi', count: '1 of 2 in picker', models, columns, defaultLabel: 'Default for chats', empty: 'None', open: true, onshow });
		await page.getByRole('switch', { name: 'gemini-pro' }).click();
		expect(onshow).toHaveBeenCalledWith('admin:a2', true);
	});
});

describe('ConnectRow and WhoPays', () => {
	it('offers a few providers to start with, and all of them', async () => {
		const onpick = vi.fn();
		const onall = vi.fn();
		await render(ConnectRow, { title: 'Connect a provider', detail: 'API key, or a server on your machine', suggestions: [{ id: 'anthropic', name: 'Anthropic' }], all: 'All 74', onpick, onall });
		expect(box(document.querySelector('div')!).height).toBe(57);
		await page.getByRole('button', { name: 'Anthropic' }).click();
		await page.getByRole('button', { name: 'All 74' }).click();
		expect([onpick.mock.calls, onall.mock.calls.length]).toEqual([[['anthropic']], 1]);
	});

	it('says who pays, Included in the accent', async () => {
		await render(WhoPays, { label: 'Who pays', kinds: [{ tag: 'Included', text: 'Counts toward your plan.', accent: true }, { tag: 'Your key', text: 'Billed by that provider.' }] });
		expect(getComputedStyle(page.getByText('Included').element()).color).toBe(tokenColor('nm-accent'));
		expect(box(page.getByText('Included').element()).height).toBe(20);
	});
});

describe('the Connect dialog', () => {
	it('is 900 × 620: the providers in a 320 column, then connecting the one chosen', async () => {
		await page.viewport(1512, 1000);
		const onpick = vi.fn();
		await render(ConnectFixture, { onpick });
		const dialog = document.querySelector('.shadow-float')!;
		expect([box(dialog).width, box(dialog).height]).toEqual([900, 620]);
		expect(box(dialog.firstElementChild!).width).toBe(320);
		await expect.element(page.getByRole('button', { name: 'Anthropic' })).toHaveAttribute('aria-current', 'true');
		await page.getByRole('button', { name: /Custom endpoint/ }).click();
		expect(onpick).toHaveBeenCalledWith('custom');
	});

	it('hides the key until asked, says it works, and adds the chosen models', async () => {
		const onaction = vi.fn();
		await render(ConnectFixture, { onaction });
		const key = page.getByLabelText('API key');
		await expect.element(key).toHaveAttribute('type', 'password');
		await page.getByRole('button', { name: 'Show the key' }).click();
		await expect.element(key).toHaveAttribute('type', 'text');
		expect(getComputedStyle(page.getByText('It works · 2 models found').element()).color).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByRole('switch', { name: 'claude-sonnet-5-5' })).toHaveAttribute('aria-checked', 'true');
		await page.getByRole('button', { name: 'Add 1 model' }).click();
		expect(onaction).toHaveBeenCalledOnce();
	});
});

describe('Select, tall', () => {
	it('is 34 on the surface with its mascot before the value', async () => {
		const lead = createRawSnippet(() => ({ render: () => '<span>N</span>' }));
		await render(Select, { label: 'New chats start with', options: [{ value: 'admin:a1', label: 'deepseek-flash · Nomi' }], value: 'admin:a1', tall: true, lead });
		const select = page.getByRole('combobox', { name: 'New chats start with' }).element();
		expect([box(select).height, getComputedStyle(select).backgroundColor, getComputedStyle(select).paddingLeft]).toEqual([34, tokenBackground('nm-surface'), '34px']);
	});
});
