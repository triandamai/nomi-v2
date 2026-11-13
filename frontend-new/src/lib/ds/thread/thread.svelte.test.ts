import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { MessageSquare, RotateCcw } from '@lucide/svelte';
import Prose from './Prose.svelte';
import ChatComposer from './ChatComposer.svelte';
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
