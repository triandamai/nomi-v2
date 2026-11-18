import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { MessageSquare, RotateCcw } from '@lucide/svelte';
import Prose from './Prose.svelte';
import AgentReply from './AgentReply.svelte';
import ChatComposer from './ChatComposer.svelte';
import PhoneChatRow from './PhoneChatRow.svelte';
import PhoneComposer from './PhoneComposer.svelte';
import ChatComposerFixture from './ChatComposerFixture.svelte';
import Starters from './Starters.svelte';
import TodayStrip from './TodayStrip.svelte';
import ChatTopBar from './ChatTopBar.svelte';
import TurnNote from './TurnNote.svelte';
import ReplyBar from './ReplyBar.svelte';
import FailedReply from './FailedReply.svelte';
import ChatListRow from './ChatListRow.svelte';
import AgentReplyFixture from './AgentReplyFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The chat core's own pieces (src/lib/ds/thread): each composes the design system's components
// in the pen's layout for the chat flows (ENb07, Cf6GO, HKYo1, J3ks0, TpuRc) and Home (Vr4nn).

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const crew = [
	{ kind: 'crew' as const, agent: 'nomi' as const, name: 'Nomi', role: 'Talks with you' },
	{ kind: 'crew' as const, agent: 'dana' as const, name: 'Dana', role: 'Money and subscriptions' },
	{ kind: 'crew' as const, agent: 'kala' as const, name: 'Kala', role: 'Reminders on time' },
];

describe('Prose', () => {
	it('shows the reply the server rendered, in the pen’s body style', async () => {
		await render(Prose, { html: '<p>Rp 2,1 jt on <strong>food</strong>.</p><ul><li>GoFood</li></ul>' });
		const paragraph = page.getByText(/Rp 2,1 jt on/).element();
		const style = getComputedStyle(paragraph);
		expect(style.fontSize).toBe('15px');
		expect(style.lineHeight).toBe('24px');
		expect(style.color).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByRole('listitem')).toHaveTextContent('GoFood');
	});
});

describe('ChatComposer', () => {
	it('sends on Enter, keeps Shift+Enter for a new line, and stays off when empty', async () => {
		const onsend = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask anything…', onsend });
		const send = page.getByRole('button', { name: 'Send' });
		await expect.element(send).toBeDisabled();
		const field = page.getByRole('textbox', { name: 'Message' });
		await field.fill('How much on food?');
		await userEvent.keyboard('{Shift>}{Enter}{/Shift}');
		expect(onsend).not.toHaveBeenCalled();
		await userEvent.keyboard('{Enter}');
		expect(onsend).toHaveBeenCalledWith('How much on food?');
		await expect.element(field).toHaveValue('');
	});

	it('with Enter sends off, Enter makes a new line and ⌘ or Ctrl+Enter sends', async () => {
		const onsend = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask anything…', onsend, enterSends: false, hints: true });
		const field = page.getByRole('textbox', { name: 'Message' });
		await field.fill('Two lines');
		await userEvent.keyboard('{Enter}');
		expect(onsend).not.toHaveBeenCalled();
		await expect.element(field).toHaveValue('Two lines\n');
		await userEvent.keyboard('{Control>}{Enter}{/Control}');
		expect(onsend).toHaveBeenCalledWith('Two lines');
		await expect.element(page.getByText('⌘↵')).toBeInTheDocument();
	});

	it('lays out as the pen draws it: padded [14,12,10,12] inside its stroke, 14 between field and toolbar', async () => {
		await render(ChatComposer, { placeholder: 'Ask anything…', onsend: () => {}, model: 'deepseek-flash', models: [{ group: 'Nomi credits', models: [{ key: 'admin:1', label: 'deepseek-flash', source: 'Nomi credits', chosen: true }] }] });
		const box = document.querySelector<HTMLElement>('[data-composer]')!;
		const style = getComputedStyle(box);
		expect([style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft, style.borderTopWidth]).toEqual(['13px', '11px', '9px', '11px', '1px']);
		expect(style.rowGap).toBe('14px');
		expect(style.borderRadius).toBe('10px');
	});

	it('opens the crew picker on @, moves with the arrows and picks with Enter', async () => {
		const onsend = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask', crew, onsend });
		const field = page.getByRole('textbox', { name: 'Message' });
		await field.click();
		await userEvent.keyboard('How much? @');
		await expect.element(page.getByRole('listbox', { name: 'Mention someone or a file' })).toBeVisible();
		await userEvent.keyboard('d');
		await expect.poll(() => page.getByRole('option').elements().length).toBe(1);
		await userEvent.keyboard('{Enter}');
		await expect.element(field).toHaveValue('How much? @Dana ');
		await expect.element(page.getByRole('listbox')).not.toBeInTheDocument();
		await userEvent.keyboard('{Enter}');
		expect(onsend).toHaveBeenCalledWith('How much? @Dana');
	});

	it('closes the picker with Escape and leaves the text', async () => {
		await render(ChatComposer, { placeholder: 'Ask', crew, onsend: () => {} });
		await page.getByRole('textbox').click();
		await userEvent.keyboard('@');
		await expect.poll(() => page.getByRole('option').elements().length).toBe(3);
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('option', { name: /Dana/ })).toHaveAttribute('aria-selected', 'true');
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('listbox')).not.toBeInTheDocument();
		await expect.element(page.getByRole('textbox')).toHaveValue('@');
	});

	it('turns Send into Stop while the crew answers', async () => {
		const onstop = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask', busy: true, onstop, onsend: () => {} });
		await expect.element(page.getByRole('button', { name: 'Send' })).not.toBeInTheDocument();
		const stop = page.getByRole('button', { name: 'Stop reply' }).first();
		await expect.element(stop).toHaveAttribute('aria-keyshortcuts', 'Escape');
		expect(getComputedStyle(stop.element()).backgroundColor).toBe(tokenBackground('nm-text'));
		await stop.click();
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('picks who it goes to, the model and the thinking level from their chips', async () => {
		const onto = vi.fn();
		const onmodel = vi.fn();
		const onthinking = vi.fn();
		await render(ChatComposer, {
			placeholder: 'Ask',
			onsend: () => {},
			to: { agent: 'nomi', name: 'Nomi' },
			crew,
			onto,
			model: 'deepseek-flash',
			models: [
				{ group: 'Your keys', models: [{ key: 'conn:c:sonnet', label: 'claude-sonnet', source: 'Anthropic', chosen: false }] },
				{ group: 'Nomi credits', models: [{ key: 'admin:1', label: 'deepseek-flash', source: 'Nomi credits', chosen: true }] },
			],
			onmodel,
			thinking: { label: 'Default', value: 'medium', levels: [{ value: 'low', label: 'Low' }, { value: 'medium', label: 'Default' }] },
			onthinking,
		});
		await page.getByRole('button', { name: 'Nomi' }).click();
		await page.getByRole('menuitem', { name: 'Dana' }).click();
		expect(onto).toHaveBeenCalledWith('Dana');
		await page.getByRole('button', { name: 'deepseek-flash' }).click();
		await page.getByRole('menuitem', { name: /claude-sonnet/ }).click();
		expect(onmodel).toHaveBeenCalledWith('conn:c:sonnet');
		await page.getByRole('button', { name: 'Default' }).click();
		await page.getByRole('menuitem', { name: 'Low' }).click();
		expect(onthinking).toHaveBeenCalledWith('low');
	});

	it('draws no chip with nothing to offer', async () => {
		await render(ChatComposer, { placeholder: 'Ask', onsend: () => {}, model: 'deepseek-flash', models: [] });
		await expect.element(page.getByRole('button', { name: 'deepseek-flash' })).not.toBeInTheDocument();
	});
});

describe('Starters', () => {
	it('lists each starter with who it goes to, and asks it when picked', async () => {
		const onpick = vi.fn();
		await render(Starters, {
			items: [
				{ agent: 'dana', label: 'Summarize this week’s spending', route: '→ dana' },
				{ agent: 'rena', label: 'Plan my Saturday', route: '→ rena' },
			],
			onpick,
		});
		await expect.element(page.getByText('Try')).toBeVisible();
		await page.getByRole('button', { name: /Plan my Saturday/ }).click();
		expect(onpick).toHaveBeenCalledWith(1);
	});

	it('is nothing without starters', async () => {
		await render(Starters, { items: [], onpick: () => {} });
		await expect.element(page.getByText('Try')).not.toBeInTheDocument();
	});
});

describe('TodayStrip', () => {
	it('heads its three columns with a count, a line and a way on', async () => {
		const onclick = vi.fn();
		await render(TodayStrip, {
			updated: '08:14',
			columns: [
				{ key: 'out', title: 'While you were out', count: 0, body: 'All quiet.', action: { label: 'Open notifications', href: '/inbox' } },
				{ key: 'reminders', title: 'Reminders', count: 2, body: 'Stretch · 16:00', action: { label: 'Add reminder', onclick } },
				{ key: 'plans', title: 'Plans in progress', count: 0, body: 'No plans running.', action: { label: 'Start a plan', onclick } },
			],
		});
		await expect.element(page.getByRole('heading', { name: 'Today' })).toBeVisible();
		await expect.element(page.getByText('updated 08:14')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'Open notifications' })).toHaveAttribute('href', '/inbox');
		await page.getByRole('button', { name: 'Add reminder' }).click();
		expect(onclick).toHaveBeenCalledOnce();
		const column = page.getByRole('article').first().element();
		expect(getComputedStyle(column).paddingTop).toBe('16px');
	});
});

describe('ChatTopBar', () => {
	it('names the chat, and opens its menu', async () => {
		const onpick = vi.fn();
		await render(ChatTopBar, { title: 'Food spending in October', meta: 'chat_7f3a2c', onrename: () => {}, oncancelrename: () => {}, onpick });
		await expect.element(page.getByText('Food spending in October')).toBeVisible();
		await expect.element(page.getByText('chat_7f3a2c')).toBeVisible();
		await page.getByRole('button', { name: 'Chat options' }).click();
		await page.getByRole('menuitem', { name: /Pin to sidebar/ }).click();
		expect(onpick).toHaveBeenCalledWith('pin');
	});

	it('renames in place: Enter saves, Escape cancels', async () => {
		const onrename = vi.fn();
		const oncancelrename = vi.fn();
		await render(ChatTopBar, { title: 'Starting a new conversation', renaming: true, onrename, oncancelrename, onpick: () => {} });
		const field = page.getByRole('textbox', { name: 'Chat name' });
		await expect.element(field).toHaveValue('Starting a new conversation');
		await field.fill('October food check');
		await userEvent.keyboard('{Enter}');
		expect(onrename).toHaveBeenCalledWith('October food check');
		await userEvent.keyboard('{Escape}');
		expect(oncancelrename).toHaveBeenCalled();
	});
});

describe('TurnNote', () => {
	it('is a quiet mono line, or a bordered row with an action', async () => {
		const onclick = vi.fn();
		await render(TurnNote, { icon: RotateCcw, text: 'sent after you signed in · 10:31' });
		const line = page.getByText('sent after you signed in · 10:31').element();
		expect(getComputedStyle(line).fontFamily).toContain('Geist Mono');
		document.body.innerHTML = '';
		await render(TurnNote, { icon: RotateCcw, kind: 'box', text: 'First try failed.', action: { label: 'Details', onclick } });
		await page.getByRole('button', { name: 'Details' }).click();
		expect(onclick).toHaveBeenCalledOnce();
		expect(getComputedStyle(page.getByText('First try failed.').element().parentElement!).paddingTop).toBe('8px');
	});
});

describe('AgentReply', () => {
	it('says who answered and when, with the version arrows, its parts 16 apart', async () => {
		const onprev = vi.fn();
		await render(AgentReplyFixture, { version: { current: 2, total: 2, onprev, onnext: () => {} } });
		await expect.element(page.getByText('Dana')).toBeVisible();
		await expect.element(page.getByText('14:05 · 9s')).toBeVisible();
		await page.getByRole('button', { name: 'Previous version' }).click();
		expect(onprev).toHaveBeenCalledOnce();
		expect(getComputedStyle(page.getByRole('article').element()).rowGap).toBe('16px');
	});

	it('dims while the message it answers is edited', async () => {
		await render(AgentReplyFixture, { dimmed: true });
		expect(getComputedStyle(page.getByRole('article').element()).opacity).toBe('0.4');
	});

	it('dims to 0.35 on a phone, as Message controls draws it (bRx3K)', async () => {
		await render(AgentReplyFixture, { dimmed: true, phone: true });
		expect(getComputedStyle(page.getByRole('article').element()).opacity).toBe('0.35');
	});
});

describe('ReplyBar', () => {
	it('likes in one tap and takes it back on the next', async () => {
		const onfeedback = vi.fn(async () => true);
		await render(ReplyBar, { text: 'Rp 2,1 jt', onfeedback, ondislike: async () => true });
		const like = page.getByRole('button', { name: 'Good reply' });
		await like.click();
		await expect.element(like).toHaveAttribute('aria-pressed', 'true');
		await like.click();
		await expect.element(like).toHaveAttribute('aria-pressed', 'false');
		expect(onfeedback.mock.calls).toEqual([['up'], [null]]);
	});

	it('puts the like back when the server says no', async () => {
		await render(ReplyBar, { text: 'x', onfeedback: async () => false, ondislike: async () => true });
		const like = page.getByRole('button', { name: 'Good reply' });
		await like.click();
		await expect.element(like).toHaveAttribute('aria-pressed', 'false');
	});

	it('asks why after a dislike, and thanks once it’s sent', async () => {
		const ondislike = vi.fn(async () => true);
		await render(ReplyBar, { text: 'x', onfeedback: async () => true, ondislike });
		await page.getByRole('button', { name: 'Bad reply' }).click();
		await page.getByRole('button', { name: 'Not accurate' }).click();
		await page.getByRole('button', { name: 'Send feedback' }).click();
		expect(ondislike).toHaveBeenCalledWith({ reasons: ['inaccurate'], note: '', includeChat: false });
		await expect.element(page.getByText('Thanks. Feedback sent with this reply only.')).toBeVisible();
	});

	it('regenerates in a style from its menu, and closes it with Escape', async () => {
		const onregenerate = vi.fn();
		await render(ReplyBar, { text: 'x', onfeedback: async () => true, ondislike: async () => true, onregenerate });
		await page.getByRole('button', { name: 'Regenerate' }).click();
		await expect.element(page.getByRole('menuitem', { name: /Try again/ })).toHaveFocus();
		await expect.element(page.getByRole('menuitem', { name: /another model/ })).not.toBeInTheDocument();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitem', { name: 'Shorter' })).toHaveFocus();
		await userEvent.keyboard('{Enter}');
		expect(onregenerate).toHaveBeenCalledWith('shorter');
		await page.getByRole('button', { name: 'Regenerate' }).click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
	});
});

describe('FailedReply', () => {
	it('says what happened in the backend’s words, then names it with the next moves', async () => {
		const onclick = vi.fn();
		await render(FailedReply, { text: 'DeepSeek is having trouble right now.', title: 'Provider unavailable', actions: [{ label: 'Retry', icon: RotateCcw, onclick }] });
		await expect.element(page.getByText('DeepSeek is having trouble right now.')).toBeVisible();
		await expect.element(page.getByRole('alert')).toHaveTextContent('Provider unavailable');
		await page.getByRole('button', { name: 'Retry' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('ChatListRow', () => {
	it('links to the chat, and pins or deletes it from the row', async () => {
		const onpin = vi.fn();
		const ondelete = vi.fn();
		await render(ChatListRow, { href: '/chat/a', title: 'Budget belanja Oktober', preview: 'Rp 2,1 jt', time: '07:00', pinned: true, onpin, ondelete });
		await expect.element(page.getByRole('link', { name: 'Budget belanja Oktober' })).toHaveAttribute('href', '/chat/a');
		await expect.element(page.getByText('Pinned')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Unpin from sidebar' }).click();
		await page.getByRole('button', { name: 'Delete chat' }).click();
		expect(onpin).toHaveBeenCalledOnce();
		expect(ondelete).toHaveBeenCalledOnce();
		expect(MessageSquare).toBeTruthy();
	});
});

describe('Send, then Stop in its place', () => {
	it('a double-click on Send sends once and doesn’t stop the reply it started', async () => {
		const onsend = vi.fn();
		const onstop = vi.fn();
		await render(ChatComposerFixture, { onsend, onstop });
		await userEvent.dblClick(page.getByRole('button', { name: 'Send' }));
		expect(onsend).toHaveBeenCalledOnce();
		expect(onstop).not.toHaveBeenCalled();
		// A click of its own, a moment later, stops it.
		await new Promise((resolve) => setTimeout(resolve, 600));
		await page.getByRole('button', { name: 'Stop reply' }).last().click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('PhoneChatRow', () => {
	it('is the pen’s phone row: [12,20], a 40 avatar, the title at 16 and the last line at 14', async () => {
		await page.viewport(390, 844);
		await render(PhoneChatRow, { href: '/chat/1', title: 'Food spending in October', time: '09:41', preview: 'Rp 2,1 jt on food', agent: 'dana' });
		const row = page.getByRole('link', { name: /Food spending in October/ }).element();
		const style = getComputedStyle(row);
		expect([row.getBoundingClientRect().width, style.paddingTop, style.paddingLeft, style.columnGap]).toEqual([390, '12px', '20px', '12px']);
		expect(row.querySelector('span')!.getBoundingClientRect().width).toBe(40);
		expect(getComputedStyle(page.getByText('Food spending in October').element()).fontSize).toBe('16px');
		expect(getComputedStyle(page.getByText('Rp 2,1 jt on food').element()).fontSize).toBe('14px');
	});

	it('wears the chat glyph where who answered isn’t known, and says when the crew is at work', async () => {
		await render(PhoneChatRow, { href: '/chat/1', title: 'Weekly groceries', time: '17:02', working: 'working' });
		await expect.element(page.getByText('working')).toBeVisible();
		expect(page.getByText('17:02').elements()).toHaveLength(0);
	});

	it('opens its sheet on a long press or its More, and a long press isn’t also a tap', async () => {
		const onmore = vi.fn();
		await render(PhoneChatRow, { href: '#chat', title: 'Weekly groceries', time: '17:02', more: 'More for Weekly groceries', onmore });
		const row = page.getByRole('link', { name: /Weekly groceries/ });
		row.element().dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
		await new Promise((resolve) => setTimeout(resolve, 600));
		row.element().dispatchEvent(new PointerEvent('pointerup', { bubbles: true }));
		expect(onmore).toHaveBeenCalledOnce();
		const click = new MouseEvent('click', { bubbles: true, cancelable: true });
		row.element().dispatchEvent(click);
		expect(click.defaultPrevented).toBe(true);
		// For the keyboard: More shows once it has focus.
		const more = page.getByRole('button', { name: 'More for Weekly groceries' });
		(more.element() as HTMLElement).focus();
		await expect.element(more).toBeVisible();
		await userEvent.keyboard('{Enter}');
		expect(onmore).toHaveBeenCalledTimes(2);
	});
});

describe('PhoneComposer', () => {
	const crew = [
		{ kind: 'crew' as const, agent: 'dana' as const, name: 'Dana', role: 'Money and subscriptions' },
		{ kind: 'crew' as const, agent: 'kala' as const, name: 'Kala', role: 'Reminders on time' },
	];

	it('is the pen’s pill: 16pt, Send only once there’s text, the toolbar only while writing', async () => {
		await page.viewport(390, 844);
		const onsend = vi.fn();
		await render(PhoneComposer, { placeholder: 'Message Nomi', onsend, model: 'deepseek-flash', onmodel: () => {}, oncrew: () => {} });
		const field = page.getByRole('textbox', { name: 'Message' });
		expect(getComputedStyle(field.element()).fontSize).toBe('16px');
		const bar = document.querySelector<HTMLElement>('[data-phone-composer]')!;
		expect([getComputedStyle(bar).paddingLeft, getComputedStyle(bar).borderTopWidth]).toEqual(['12px', '1px']);
		expect(page.getByRole('button', { name: 'Send' }).elements()).toHaveLength(0);
		expect(page.getByRole('button', { name: 'Model: deepseek-flash' }).elements()).toHaveLength(0);
		await field.click();
		await expect.element(page.getByRole('button', { name: 'Model: deepseek-flash' })).toBeVisible();
		await field.fill('Berapa belanja makan Oktober?');
		await page.getByRole('button', { name: 'Send' }).click();
		expect(onsend).toHaveBeenCalledWith('Berapa belanja makan Oktober?');
		await expect.element(field).toHaveValue('');
	});

	it('picks a crew member after "@"', async () => {
		await render(PhoneComposer, { placeholder: 'Message Nomi', onsend: () => {}, crew });
		const field = page.getByRole('textbox', { name: 'Message' });
		await field.click();
		await userEvent.keyboard('Berapa makan? @da');
		await expect.element(page.getByRole('option', { name: /Dana/ })).toBeVisible();
		await userEvent.keyboard('{Enter}');
		await expect.element(field).toHaveValue('Berapa makan? @Dana ');
	});

	it('doesn’t stop the reply it just sent on the second tap of a double tap', async () => {
		const onsend = vi.fn();
		const onstop = vi.fn();
		// The guard's clock is held still, so a slow machine can't make the second tap “late”.
		let now = 1000;
		const clock = vi.spyOn(performance, 'now').mockImplementation(() => now);
		try {
			const screen = await render(PhoneComposer, { placeholder: 'Message Nomi', value: 'Halo', onsend, onstop });
			const send = page.getByRole('button', { name: 'Send' });
			await send.click();
			await screen.rerender({ placeholder: 'Message Nomi', streaming: true, onsend, onstop });
			now += 100;
			await page.getByRole('button', { name: 'Stop reply' }).click();
			expect(onstop).not.toHaveBeenCalled();
			now += 500;
			await page.getByRole('button', { name: 'Stop reply' }).click();
			expect(onstop).toHaveBeenCalledOnce();
		} finally {
			clock.mockRestore();
		}
	});
});

describe('a reply on a phone', () => {
	it('sets the prose at 16 on 1.55 in text, and the reply’s parts 12 apart under the mobile byline', async () => {
		await page.viewport(390, 844);
		await render(Prose, { html: '<p>Rp 2,1 jt on food in October.</p>', phone: true });
		const prose = document.querySelector<HTMLElement>('.prose')!;
		expect([getComputedStyle(prose).fontSize, getComputedStyle(prose).lineHeight, getComputedStyle(prose).color]).toEqual(['16px', '25px', tokenColor('nm-text')]);
		document.body.innerHTML = '';
		await render(AgentReply, { agent: 'dana', name: 'Dana', meta: '14:02 · 9s', phone: true });
		const article = page.getByRole('article', { name: 'Dana' }).element();
		expect(getComputedStyle(article).rowGap).toBe('12px');
		expect(getComputedStyle(page.getByText('Dana', { exact: true }).element()).fontSize).toBe('14px');
	});
});

