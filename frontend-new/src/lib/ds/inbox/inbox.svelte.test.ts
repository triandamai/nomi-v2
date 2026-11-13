import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Calendar, Clock, Repeat } from '@lucide/svelte';
import SourceTile from './SourceTile.svelte';
import InboxHeading from './InboxHeading.svelte';
import NowLine from './NowLine.svelte';
import EarlierToggle from './EarlierToggle.svelte';
import Agenda from './Agenda.svelte';
import PreferenceRow from './PreferenceRow.svelte';
import QuickReminder from './QuickReminder.svelte';
import InboxFixture from './InboxFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The Inbox's pieces (App / Inbox BFoiZ): the feed's cards and rows, the now line, the section
// headings, and the Coming up column's quick reminder, agenda and email settings.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('SourceTile', () => {
	it('is the crew member’s tile with their mascot: 32 with corners of 8, or 26 with 6.5', async () => {
		await render(SourceTile, { agent: 'kala' });
		const tile = document.querySelector('[data-source-tile]')!;
		expect([box(tile).width, box(tile).height, getComputedStyle(tile).borderRadius]).toEqual([32, 32, '8px']);
		expect(getComputedStyle(tile).backgroundColor).toBe(tokenBackground('crew-kala-bg'));
		expect(box(tile.querySelector('svg')!).width).toBe(20);
		document.body.innerHTML = '';
		await render(SourceTile, { agent: 'dana', size: 26 });
		const small = document.querySelector('[data-source-tile]')!;
		expect([box(small).width, getComputedStyle(small).borderRadius]).toEqual([26, '6.5px']);
	});
});

describe('InboxHeading', () => {
	it('names a section in mono capitals, its count quieter, 6 above the list', async () => {
		await render(InboxHeading, { label: 'Needs you', count: 3 });
		const heading = page.getByRole('heading', { name: /Needs you/ }).element();
		expect(box(heading).height).toBe(20);
		const label = page.getByText('Needs you').element();
		expect([getComputedStyle(label).textTransform, getComputedStyle(label).color]).toEqual(['uppercase', tokenColor('nm-text-2')]);
		expect(getComputedStyle(page.getByText('3').element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('the feed’s cards and rows', () => {
	it('draws what needs the person as a card: 16 across, 14 down, the unread dot at the end', async () => {
		await render(InboxFixture);
		const card = page.getByRole('article', { name: 'Deploy failed: invoice-sinar-jaya' }).element();
		const style = getComputedStyle(card);
		expect([style.paddingTop, style.paddingLeft, style.borderRadius]).toEqual(['14px', '16px', '10px']);
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(box(card).height).toBe(103);
		const dot = card.querySelector('[data-unread]')!;
		expect([box(dot).width, box(card).right - box(dot).right]).toEqual([6, 16]);
		await expect.element(page.getByRole('button', { name: 'Open log' })).toBeVisible();
	});

	it('draws today’s entries as 56px rows, the past ones’ times quieter and a finished one dimmed', async () => {
		await render(InboxFixture);
		const gym = page.getByTestId('row-gym').element();
		const stretch = page.getByTestId('row-stretch').element();
		expect([box(gym).height, box(stretch).height]).toEqual([56, 56]);
		expect(getComputedStyle(gym).opacity).toBe('0.55');
		expect(getComputedStyle(page.getByText('06:30').element()).color).toBe(tokenColor('nm-text-3'));
		expect(getComputedStyle(page.getByText('16:00').element()).color).toBe(tokenColor('nm-text'));
		await expect.element(page.getByRole('button', { name: 'Mark done' })).toBeVisible();
	});

	it('marks now between them, in the accent', async () => {
		await render(NowLine, { time: '08:14', label: 'now' });
		const line = page.getByRole('separator', { name: 'now 08:14' }).element();
		expect(box(line).height).toBe(22);
		expect(getComputedStyle(line).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		expect(getComputedStyle(page.getByText('08:14').element()).color).toBe(tokenColor('nm-accent'));
	});

	it('folds the earlier ones away until asked', async () => {
		const ontoggle = vi.fn();
		await render(EarlierToggle, { label: 'Earlier', count: 5, ontoggle });
		const toggle = page.getByRole('button', { name: /Earlier/ });
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'false');
		expect(box(toggle.element()).height).toBe(18);
		await toggle.click();
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'true');
		expect(ontoggle).toHaveBeenCalledWith(true);
	});
});

describe('QuickReminder', () => {
	it('shows what it read from the line as chips, and sends on Enter', async () => {
		const onsubmit = vi.fn();
		await render(QuickReminder, {
			label: 'Quick reminder',
			placeholder: 'Pay rent every 25th at 10am',
			value: '',
			chips: [
				{ icon: Calendar, label: 'Sun 25 Oct' },
				{ icon: Clock, label: '10:00' },
				{ icon: Repeat, label: 'monthly' },
			],
			hint: '↵ add',
			onsubmit,
		});
		const field = page.getByRole('textbox', { name: 'Quick reminder' });
		await field.fill('pay PLN electricity every 25th at 10am');
		await userEvent.keyboard('{Enter}');
		expect(onsubmit).toHaveBeenCalledOnce();
		const frame = document.querySelector('[data-quick-frame]')!;
		expect(box(frame).height).toBe(72);
		const chip = page.getByText('Sun 25 Oct').element().parentElement!;
		expect([box(chip).height, getComputedStyle(chip).backgroundColor]).toEqual([22, tokenBackground('nm-surface-2')]);
	});

	it('says what’s wrong under the field', async () => {
		await render(QuickReminder, { label: 'Quick reminder', value: 'rent', chips: [], hint: 'add a time', error: 'Say when, like “at 10am”.' });
		await expect.element(page.getByRole('textbox', { name: 'Quick reminder' })).toHaveAttribute('aria-invalid', 'true');
		await expect.element(page.getByText('Say when, like “at 10am”.')).toBeVisible();
	});
});

describe('Agenda', () => {
	it('lists the days ahead, today’s name stronger, each entry with its tile and time', async () => {
		await render(Agenda, {
			label: 'Next 7 days',
			count: 3,
			empty: 'Nothing in the next 7 days.',
			days: [
				{ label: 'Today', today: true, items: [{ agent: 'kala', title: 'Stretch', time: '16:00' }, { agent: 'kala', title: 'Call Siti', time: '20:00' }] },
				{ label: 'Thu 22', items: [{ agent: 'dana', title: 'Netflix renews · Rp 186.000', time: '09:00' }] },
			],
		});
		expect(getComputedStyle(page.getByText('Today').element()).color).toBe(tokenColor('nm-text'));
		expect(getComputedStyle(page.getByText('Thu 22').element()).color).toBe(tokenColor('nm-text-3'));
		const today = page.getByRole('list').first().element();
		expect(box(today).height).toBe(58);
		await expect.element(page.getByRole('listitem').nth(2)).toHaveTextContent('Netflix renews · Rp 186.000 09:00');
	});

	it('says plainly when nothing is coming up', async () => {
		await render(Agenda, { label: 'Next 7 days', count: 0, empty: 'Nothing in the next 7 days.', days: [] });
		await expect.element(page.getByText('Nothing in the next 7 days.')).toBeVisible();
	});
});

describe('PreferenceRow', () => {
	it('is a setting with what it means, and a switch named for it', async () => {
		const onchange = vi.fn();
		await render(PreferenceRow, { title: 'Email account changes', description: 'Invites, plan and billing. Never promos.', checked: true, onchange });
		const toggle = page.getByRole('switch', { name: 'Email account changes' });
		await expect.element(toggle).toHaveAttribute('aria-checked', 'true');
		await toggle.click();
		expect(onchange).toHaveBeenCalledWith(false);
		expect(box(page.getByText('Email account changes').element().parentElement!.parentElement!).height).toBe(35);
	});
});
