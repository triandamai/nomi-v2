import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Egg, FileSpreadsheet, Fish } from '@lucide/svelte';
import DataTable from './DataTable.svelte';
import Cart from './Cart.svelte';
import CodeBlock from './CodeBlock.svelte';
import Diff from './Diff.svelte';
import Hyperlink from './Hyperlink.svelte';
import Citation from './Citation.svelte';
import Sources from './Sources.svelte';
import LinkPreview from './LinkPreview.svelte';
import MentionCrew from './MentionCrew.svelte';
import MentionFile from './MentionFile.svelte';
import MentionPicker from './MentionPicker.svelte';
import Reminder from './Reminder.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Rich content: structure only when the content has it. Tables for comparable
// rows, cards for things you act on, code and diffs in mono.

afterEach(() => {
	vi.restoreAllMocks();
});

const mono = (element: Element) => getComputedStyle(element).fontFamily;

describe('DataTable', () => {
	const props = {
		title: 'Food spending by week',
		columns: ['Week', 'Groceries', 'GoFood', 'Total', 'vs Sep'],
		rows: [
			['1–7 Oct', '312.000', '168.000', '480.000', { value: '+12%', tone: 'bad' as const }],
			['22–28 Oct', '274.000', '196.000', '470.000', { value: '−6%', tone: 'good' as const }],
		],
		total: ['October', '1.173.000', '927.000', '2.100.000', { value: '+20%', tone: 'bad' as const }],
		source: 'Rp · from Dana · 142 transactions',
		link: { href: 'https://docs.google.com/spreadsheets/d/1', label: 'Open in Sheets' },
	};

	it('is a real table named by its title, numbers in mono and the total row last', async () => {
		await render(DataTable, props);
		const table = page.getByRole('table', { name: 'Food spending by week' });
		await expect.element(table).toBeVisible();
		expect(page.getByRole('columnheader').elements()).toHaveLength(5);
		expect(page.getByRole('row').elements()).toHaveLength(4);
		expect(mono(page.getByText('312.000').element())).toContain('Geist Mono');
		expect(getComputedStyle(page.getByText('1–7 Oct').element()).fontFamily).not.toContain('Mono');
		expect(getComputedStyle(page.getByText('312.000').element()).textAlign).toBe('right');
		expect(getComputedStyle(page.getByText('+12%').element()).color).toBe(tokenColor('nm-danger'));
		expect(getComputedStyle(page.getByText('−6%').element()).color).toBe(tokenColor('nm-accent'));
		const total = page.getByRole('row').elements()[3];
		expect(total.textContent).toContain('October');
		expect(getComputedStyle(total).backgroundColor).toBe(tokenBackground('nm-surface'));
		await expect.element(page.getByText('Rp · from Dana · 142 transactions')).toBeVisible();
		await expect.element(page.getByRole('link', { name: /Open in Sheets/ })).toHaveAttribute('href', props.link.href);
	});

	it('copies itself as tab-separated text and says so', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		await render(DataTable, props);
		await page.getByRole('button', { name: 'Copy table' }).click();
		expect(write).toHaveBeenCalledWith(expect.stringContaining('Week\tGroceries\tGoFood\tTotal\tvs Sep\n1–7 Oct\t312.000'));
		await expect.element(page.getByRole('button', { name: 'Copied' })).toBeVisible();
	});

	it('downloads itself as a CSV named after the title', async () => {
		const created = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:table');
		vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
		const clicked = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
			expect(this.download).toBe('food-spending-by-week.csv');
		});
		await render(DataTable, props);
		await page.getByRole('button', { name: 'Download table' }).click();
		expect(created).toHaveBeenCalledOnce();
		expect(clicked).toHaveBeenCalledOnce();
		const blob = created.mock.calls[0][0] as Blob;
		expect(await blob.text()).toContain('Week,Groceries,GoFood,Total,vs Sep');
	});
});

describe('Cart', () => {
	const items = [
		{ name: 'Telur ayam negeri', detail: '1 tray · 10 butir', icon: Egg, quantity: 2, price: '58.000' },
		{ name: 'Ikan kembung', icon: Fish, quantity: 0, price: '—', available: false, swap: 'Swap for tongkol' },
	];

	it('lists what is in it with real quantities and prices, and the totals', async () => {
		await render(Cart, {
			title: 'Weekly groceries',
			store: 'Sayurbox',
			items,
			subtotal: '124.500',
			delivery: '10.000',
			total: 'Rp 134.500',
			note: 'Prices from Sayurbox at 14:05. Dana records it when you pay.',
			checkoutHref: 'https://sayurbox.com/cart',
		});
		await expect.element(page.getByText('Sayurbox · 2 items')).toBeVisible();
		expect(page.getByRole('listitem').elements()).toHaveLength(2);
		expect(mono(page.getByText('58.000').element())).toContain('Geist Mono');
		await expect.element(page.getByText('Subtotal')).toBeVisible();
		await expect.element(page.getByText('Rp 134.500')).toBeVisible();
		const checkout = page.getByRole('link', { name: 'Checkout on Sayurbox' });
		await expect.element(checkout).toHaveAttribute('href', 'https://sayurbox.com/cart');
		await expect.element(checkout).toHaveAttribute('target', '_blank');
	});

	it('changes a quantity through its stepper', async () => {
		const onquantity = vi.fn();
		await render(Cart, { title: 'Weekly groceries', store: 'Sayurbox', items, subtotal: '58.000', total: 'Rp 58.000', checkoutHref: '#', onquantity });
		await expect.element(page.getByRole('group', { name: 'Quantity of Telur ayam negeri' })).toHaveTextContent('2');
		await page.getByRole('button', { name: 'More Telur ayam negeri' }).click();
		expect(onquantity).toHaveBeenLastCalledWith(0, 3);
		await page.getByRole('button', { name: 'Fewer Telur ayam negeri' }).click();
		expect(onquantity).toHaveBeenLastCalledWith(0, 1);
	});

	it('says plainly when an item is out of stock and offers the swap', async () => {
		const onswap = vi.fn();
		await render(Cart, { title: 'Weekly groceries', store: 'Sayurbox', items, subtotal: '58.000', total: 'Rp 58.000', checkoutHref: '#', onswap, onsave: () => {} });
		expect(getComputedStyle(page.getByText('out of stock').element()).color).toBe(tokenColor('nm-warning'));
		await expect.element(page.getByRole('button', { name: 'More Ikan kembung' })).toBeDisabled();
		await page.getByRole('button', { name: 'Swap for tongkol' }).click();
		expect(onswap).toHaveBeenCalledWith(1);
		await expect.element(page.getByRole('button', { name: 'Save as list' })).toBeVisible();
	});
});

describe('CodeBlock', () => {
	it('shows numbered lines in mono with the syntax colours, the language and the file', async () => {
		const { container } = await render(CodeBlock, {
			language: 'typescript',
			file: 'src/lib/budget.ts',
			code: 'export function weeklyBudget() {}',
			tokens: [[{ text: 'export function ', kind: 'keyword' }, { text: 'weeklyBudget', kind: 'fn' }, { text: '() {}' }], [{ text: '// done', kind: 'comment' }]],
		});
		await expect.element(page.getByText('typescript')).toBeVisible();
		await expect.element(page.getByText('src/lib/budget.ts')).toBeVisible();
		expect(getComputedStyle(page.getByText('weeklyBudget').element()).color).toBe(tokenColor('nm-code-fn'));
		expect(getComputedStyle(page.getByText('export function').element()).color).toBe(tokenColor('nm-code-keyword'));
		expect(getComputedStyle(page.getByText('// done').element()).color).toBe(tokenColor('nm-text-3'));
		expect(mono(container.querySelector('code')!)).toContain('Geist Mono');
		const numbers = [...container.querySelectorAll('[data-line-number]')];
		expect(numbers.map((number) => number.textContent)).toEqual(['1', '2']);
		expect(numbers.every((number) => number.getAttribute('aria-hidden') === 'true')).toBe(true);
	});

	it('copies the code itself, not the line numbers', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		await render(CodeBlock, { code: 'a\nb' });
		expect(page.getByText('a', { exact: true }).elements()).toHaveLength(1);
		await page.getByRole('button', { name: 'Copy' }).click();
		expect(write).toHaveBeenCalledWith('a\nb');
		await expect.element(page.getByRole('button', { name: 'Copied' })).toBeVisible();
	});
});

describe('Diff', () => {
	const hunks = [
		{
			header: '@@ -2,4 +2,6 @@ export function weeklyBudget',
			lines: [
				{ kind: 'context' as const, old: 2, new: 2, text: 'export function weeklyBudget(total: number, weeks = 4) {' },
				{ kind: 'del' as const, old: 3, text: '  return Math.round(total / weeks)' },
				{ kind: 'add' as const, new: 3, text: '  const base = Math.round(total / weeks)' },
				{ kind: 'add' as const, new: 4, text: '  return base - 25_000' },
			],
		},
	];

	it('shows the file, what it adds and removes, and each line on its own background', async () => {
		await render(Diff, { path: 'src/lib/budget.ts', hunks });
		await expect.element(page.getByText('+2', { exact: true })).toBeVisible();
		await expect.element(page.getByText('−1', { exact: true })).toBeVisible();
		await expect.element(page.getByText('2 lines added, 1 removed')).toBeInTheDocument();
		expect(getComputedStyle(page.getByText('+2', { exact: true }).element()).color).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByText('@@ -2,4 +2,6 @@ export function weeklyBudget')).toBeVisible();
		const removed = page.getByText('return Math.round(total / weeks)').element().closest('[data-kind]')!;
		expect(removed.getAttribute('data-kind')).toBe('del');
		expect(getComputedStyle(removed).backgroundColor).toBe(tokenBackground('nm-diff-del'));
		const added = page.getByText('const base = Math.round(total / weeks)').element().closest('[data-kind]')!;
		expect(getComputedStyle(added).backgroundColor).toBe(tokenBackground('nm-diff-add'));
		expect(removed.textContent).toContain('Removed');
	});

	it('asks before it applies, naming who wants the change', async () => {
		const onapply = vi.fn();
		const onreject = vi.fn();
		await render(Diff, { path: 'src/lib/budget.ts', hunks, request: { agent: 'koda', name: 'Koda', files: 1 }, onapply, onreject });
		await expect.element(page.getByText('Koda wants to change 1 file')).toBeVisible();
		await page.getByRole('button', { name: 'Apply change' }).click();
		await page.getByRole('button', { name: 'Reject' }).click();
		expect(onapply).toHaveBeenCalledOnce();
		expect(onreject).toHaveBeenCalledOnce();
	});
});

describe('Links and sources', () => {
	it('links inline a step brighter than the sentence, as in the pen, underlined on hover', async () => {
		await render(Hyperlink, { href: '#rules', label: 'house rules Bu Sri sent' });
		const link = page.getByRole('link', { name: 'house rules Bu Sri sent' });
		await expect.element(link).toHaveAttribute('href', '#rules');
		expect(getComputedStyle(link.element()).color).toBe(tokenColor('nm-text'));
		expect(getComputedStyle(link.element()).textDecorationLine).toBe('none');
		await userEvent.hover(link);
		expect(getComputedStyle(link.element()).textDecorationLine).toBe('underline');
	});

	it('opens other sites in a new tab', async () => {
		await render(Hyperlink, { href: 'https://kosku.id', label: 'kosku.id', external: true });
		await expect.element(page.getByRole('link', { name: 'kosku.id' })).toHaveAttribute('rel', 'noopener noreferrer');
	});

	it('numbers a citation and names its source for screen readers', async () => {
		await render(Citation, { n: 1, title: 'Peraturan Kos Mawar 2026.pdf', href: '#source-1' });
		const cite = page.getByRole('link', { name: 'Source 1: Peraturan Kos Mawar 2026.pdf' });
		await expect.element(cite).toHaveTextContent('1');
		expect(mono(cite.element())).toContain('Geist Mono');
		expect(getComputedStyle(cite.element()).fontSize).toBe('10px');
		expect(getComputedStyle(cite.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
	});

	it('lists the sources in citation order, with where each came from', async () => {
		const { container } = await render(Sources, {
			id: 'answer-1',
			sources: [
				{ title: 'Peraturan Kos Mawar 2026.pdf', detail: 'attachment · read by Maya', kind: 'file' },
				{ title: 'Kos Mawar Tebet — kamar AC', detail: 'kosku.id', kind: 'web', href: 'https://kosku.id/mawar' },
			],
		});
		await expect.element(page.getByText('Sources')).toBeVisible();
		const items = page.getByRole('listitem').elements();
		expect(items).toHaveLength(2);
		expect(items[0].id).toBe('answer-1-1');
		expect(items[1].textContent).toContain('2');
		expect(container.querySelector('svg.lucide-file-text')).not.toBeNull();
		expect(container.querySelector('svg.lucide-globe')).not.toBeNull();
		await expect.element(page.getByRole('link', { name: 'Kos Mawar Tebet — kamar AC' })).toHaveAttribute('href', 'https://kosku.id/mawar');
		expect(mono(page.getByText('kosku.id').element())).toContain('Geist Mono');
	});

	it('previews a link as one card that opens the page', async () => {
		const { container } = await render(LinkPreview, {
			href: 'https://kosku.id/mawar',
			host: 'kosku.id',
			title: 'Kos Mawar Tebet — kamar AC, kamar mandi dalam',
			description: 'Rp 2.200.000 / bulan · 5 menit ke Stasiun Tebet',
			image: 'data:image/gif;base64,R0lGODlhAQABAAAAACw=',
		});
		const card = page.getByRole('link', { name: /Kos Mawar Tebet — kamar AC, kamar mandi dalam/ });
		await expect.element(card).toHaveAttribute('href', 'https://kosku.id/mawar');
		await expect.element(card).toHaveAttribute('target', '_blank');
		const image = container.querySelector('img')!;
		expect(image.getAttribute('alt')).toBe('');
		expect(image.getAttribute('loading')).toBe('lazy');
		expect(image.getAttribute('referrerpolicy')).toBe('no-referrer');
	});
});

describe('Mentions', () => {
	it('tints a crew mention in that member’s colours only', async () => {
		await render(MentionCrew, { agent: 'dana', name: 'Dana' });
		const mention = page.getByText('@Dana').element();
		expect(getComputedStyle(mention).color).toBe(tokenColor('crew-dana-ink'));
		expect(getComputedStyle(mention).backgroundColor).toBe(tokenBackground('crew-dana-bg'));
	});

	it('shows a file mention in mono with its icon', async () => {
		const { container } = await render(MentionFile, { name: 'rent-2026.xlsx', icon: FileSpreadsheet });
		expect(mono(page.getByText('rent-2026.xlsx').element())).toContain('Geist Mono');
		expect(container.querySelector('svg.lucide-file-spreadsheet')).not.toBeNull();
	});

	const options = [
		{ kind: 'crew' as const, agent: 'dana' as const, name: 'Dana', role: 'Money and subscriptions' },
		{ kind: 'crew' as const, agent: 'kala' as const, name: 'Kala', role: 'Reminders on time' },
		{ kind: 'file' as const, name: 'rent-2026.xlsx', where: 'Drive', icon: FileSpreadsheet },
	];

	it('groups the crew and the files as options, the active one marked', async () => {
		await render(MentionPicker, { id: 'mentions', options, active: 0, onselect: () => {} });
		const listbox = page.getByRole('listbox', { name: 'Mention someone or a file' });
		await expect.element(listbox).toBeVisible();
		await expect.element(page.getByRole('group', { name: 'Crew' })).toBeVisible();
		await expect.element(page.getByRole('group', { name: 'Files' })).toBeVisible();
		const all = page.getByRole('option').elements();
		expect(all).toHaveLength(3);
		expect(all[0].id).toBe('mentions-option-0');
		expect(all[0].getAttribute('aria-selected')).toBe('true');
		expect(all[1].getAttribute('aria-selected')).toBe('false');
		expect(getComputedStyle(listbox.element()).boxShadow).not.toBe('none');
	});

	it('selects with a click, without taking focus from the composer', async () => {
		const onselect = vi.fn();
		await render(MentionPicker, { id: 'mentions', options, active: 0, onselect });
		await page.getByRole('option', { name: /rent-2026.xlsx/ }).click();
		expect(onselect).toHaveBeenCalledWith(options[2]);
		await userEvent.hover(page.getByRole('option', { name: /Kala/ }));
		await expect.element(page.getByRole('option', { name: /Kala/ })).toHaveAttribute('aria-selected', 'true');
	});
});

describe('Reminder', () => {
	it('shows what and when, who keeps it, and a way to edit it', async () => {
		const onedit = vi.fn();
		const { container } = await render(Reminder, { title: 'Pay kos rent to Bu Sri', when: '25 Nov · 09:00 · every month', owner: { agent: 'kala', name: 'Kala' }, onedit });
		expect(mono(page.getByText('25 Nov · 09:00 · every month').element())).toContain('Geist Mono');
		expect(container.querySelector('svg.lucide-bell')).not.toBeNull();
		await expect.element(page.getByText('Kala', { exact: true })).toBeVisible();
		expect(page.getByRole('button', { name: 'Edit reminder' }).element().querySelector('svg')?.getAttribute('width')).toBe('14');
		await page.getByRole('button', { name: 'Edit reminder' }).click();
		expect(onedit).toHaveBeenCalledOnce();
	});

	it('turns to warning when it is due, with snooze and done', async () => {
		const onsnooze = vi.fn();
		const ondone = vi.fn();
		const { container } = await render(Reminder, { title: 'Pay kos rent to Bu Sri', when: '25 Nov 09:00 · every month', due: true, doneLabel: 'Mark paid', onsnooze, ondone });
		const when = page.getByText('due now · 25 Nov 09:00 · every month').element();
		expect(getComputedStyle(when).color).toBe(tokenColor('nm-warning'));
		expect(getComputedStyle(container.querySelector('section')!).outlineColor).toBe(tokenColor('nm-warning'));
		expect(container.querySelector('svg.lucide-bell-ring')).not.toBeNull();
		await page.getByRole('button', { name: 'Snooze 1h' }).click();
		await page.getByRole('button', { name: 'Mark paid' }).click();
		expect(onsnooze).toHaveBeenCalledOnce();
		expect(ondone).toHaveBeenCalledOnce();
	});
});
