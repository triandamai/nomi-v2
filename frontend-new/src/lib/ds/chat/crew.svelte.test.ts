import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { CalendarDays } from '@lucide/svelte';
import Handoff from './Handoff.svelte';
import Boundary from './Boundary.svelte';
import MemoryUsed from './MemoryUsed.svelte';
import MemorySaved from './MemorySaved.svelte';
import ConnectPrompt from './ConnectPrompt.svelte';
import ApprovalCard from './ApprovalCard.svelte';
import BackgroundNote from './BackgroundNote.svelte';
import CrewWorking from './CrewWorking.svelte';
import SupervisorStopped from './SupervisorStopped.svelte';
import SessionExpired from './SessionExpired.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Crew behaviour: hand-offs are visible, limits are explained with an
// alternative, memory is never silent, and long work can run without you watching.

describe('Hand-off and limits', () => {
	it('shows who handed the question to whom, and why, with a way back', async () => {
		const onback = vi.fn();
		const { container } = await render(Handoff, { from: { agent: 'nomi', name: 'Nomi' }, to: { agent: 'dana', name: 'Dana' }, reason: 'she has your transaction records', onback });
		expect(container.querySelectorAll('svg[role="img"], svg').length).toBeGreaterThanOrEqual(3);
		await expect.element(page.getByText('Nomi handed this to Dana')).toBeVisible();
		expect(getComputedStyle(page.getByText('she has your transaction records').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Ask Nomi instead' }).click();
		expect(onback).toHaveBeenCalledOnce();
	});

	it('says what it can’t do, why, and what it can do instead', async () => {
		const onclick = vi.fn();
		const { container } = await render(Boundary, {
			title: "I can't cancel subscriptions for you",
			reason: 'Nomi only records money. Here’s what I can do instead:',
			alternatives: [
				{ agent: 'kala', title: 'Remind me 3 days before the next charge', detail: 'Kala · 2 Nov, 09:00', onclick },
				{ agent: 'nomi', title: "Open Netflix's cancel page", detail: 'netflix.com/cancelplan', href: 'https://netflix.com/cancelplan', external: true },
			],
		});
		expect(container.querySelector('svg.lucide-shield')).not.toBeNull();
		await page.getByRole('button', { name: /Remind me 3 days before the next charge/ }).click();
		expect(onclick).toHaveBeenCalledOnce();
		const link = page.getByRole('link', { name: /Open Netflix's cancel page/ });
		await expect.element(link).toHaveAttribute('target', '_blank');
		expect(link.element().querySelector('svg.lucide-arrow-up-right')).not.toBeNull();
	});
});

describe('Memory and connections', () => {
	it('says when an answer used something it remembers', async () => {
		const onmanage = vi.fn();
		await render(MemoryUsed, { fact: 'Sari is vegetarian', onmanage });
		await expect.element(page.getByText('From memory: Sari is vegetarian')).toBeVisible();
		await page.getByRole('button', { name: 'Manage' }).click();
		expect(onmanage).toHaveBeenCalledOnce();
	});

	it('says when it saved something new, which you can edit or undo', async () => {
		const onedit = vi.fn();
		const onundo = vi.fn();
		const { container } = await render(MemorySaved, { fact: 'you and Sari prefer trains to driving', onedit, onundo });
		await expect.element(page.getByText('Saved to memory: you and Sari prefer trains to driving')).toBeVisible();
		expect(getComputedStyle(container.querySelector('svg.lucide-bookmark-plus')!.parentElement!).color).toBe(tokenColor('nm-accent'));
		await page.getByRole('button', { name: 'Edit' }).click();
		await page.getByRole('button', { name: 'Undo' }).click();
		expect(onedit).toHaveBeenCalledOnce();
		expect(onundo).toHaveBeenCalledOnce();
	});

	it('asks to connect a tool, saying exactly what it can do and that you can disconnect', async () => {
		const onlater = vi.fn();
		await render(ConnectPrompt, {
			agent: 'tara',
			icon: CalendarDays,
			service: 'Google Calendar',
			why: 'Tara needs it to check your weekends',
			reads: 'Read your events and free/busy times',
			alternative: 'Or tell me your free weekends',
			href: 'https://accounts.google.com/o/oauth2',
			onlater,
		});
		await expect.element(page.getByText('Connect Google Calendar')).toBeVisible();
		await expect.element(page.getByText('Nothing is created or changed without asking')).toBeVisible();
		await expect.element(page.getByText('Disconnect any time in Settings › Connections')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'Connect' })).toHaveAttribute('href', 'https://accounts.google.com/o/oauth2');
		await page.getByRole('button', { name: 'Not now' }).click();
		expect(onlater).toHaveBeenCalledOnce();
	});
});

describe('Approval paths', () => {
	it('lets you edit the draft, sent exactly as you wrote it', async () => {
		const onsave = vi.fn();
		await render(ApprovalCard, {
			agent: 'tara',
			title: "Editing Tara's email",
			status: 'editing',
			fields: [{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' }],
			body: 'Selamat siang Bu Sri.',
			approveLabel: 'Save & send',
			onsave,
			oncancel: () => {},
		});
		await expect.element(page.getByText('Editing', { exact: true })).toBeVisible();
		const draft = page.getByRole('textbox', { name: "Editing Tara's email" });
		expect(getComputedStyle(draft.element()).borderTopColor).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByText("Your edits are sent as written. Tara won't rewrite them.")).toBeVisible();
		await draft.fill('Selamat siang Bu Sri. Mohon maaf telat dua hari ya, Bu.');
		await page.getByRole('button', { name: 'Save & send' }).click();
		expect(onsave).toHaveBeenCalledWith('Selamat siang Bu Sri. Mohon maaf telat dua hari ya, Bu.');
	});
});

describe('Background work and session', () => {
	it('says you can leave while it works, and roughly how long', async () => {
		await render(BackgroundNote, { eta: '~3 min' });
		await expect.element(page.getByText("You can leave this chat. I'll post the result here and in Inbox.")).toBeVisible();
		expect(getComputedStyle(page.getByText('~3 min').element()).fontFamily).toContain('Geist Mono');
	});

	it('shows who is working and stops them all in danger', async () => {
		const onstop = vi.fn();
		await render(CrewWorking, { crew: ['nomi', 'dana', 'tara'], onstop });
		await expect.element(page.getByText('3 crew working on this chat')).toBeVisible();
		const stop = page.getByRole('button', { name: 'Stop all' });
		expect(getComputedStyle(stop.element()).color).toBe(tokenColor('nm-danger'));
		await stop.click();
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('lists what the supervisor stopped, task by task, and what was not undone', async () => {
		const onresume = vi.fn();
		const { container } = await render(SupervisorStopped, {
			meta: '14:31 · you asked',
			tasks: [
				{ agent: 'nomi', title: 'Find Whoosh trains', detail: 'done before stop', state: 'done' },
				{ agent: 'nomi', title: 'Shortlist guesthouses', detail: 'stopped at 2 of 3 found', state: 'stopped' },
				{ agent: 'tara', title: 'Share itinerary with Sari', detail: 'not started · nothing sent', state: 'waiting' },
			],
			ondiscard: () => {},
			onresume,
		});
		await expect.element(page.getByText('Supervisor stopped 3 tasks')).toBeVisible();
		expect(page.getByRole('listitem').elements()).toHaveLength(3);
		expect(container.querySelector('svg.lucide-check')).not.toBeNull();
		expect(container.querySelector('svg.lucide-square')).not.toBeNull();
		expect(container.querySelector('svg.lucide-circle-dashed')).not.toBeNull();
		await expect.element(page.getByText('Nothing that left Nomi was undone.')).toBeVisible();
		await page.getByRole('button', { name: 'Resume' }).click();
		expect(onresume).toHaveBeenCalledOnce();
	});

	it('says you were signed out and that your draft is kept', async () => {
		await render(SessionExpired, { draft: 'Find Whoosh trains for Sat 1 Nov…', signInHref: '/sign-in?next=/chat/1' });
		await expect.element(page.getByRole('heading', { name: "You've been signed out" })).toBeVisible();
		await expect.element(page.getByText('draft kept')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'Sign in again' })).toHaveAttribute('href', '/sign-in?next=/chat/1');
		expect(getComputedStyle(page.getByText('Find Whoosh trains for Sat 1 Nov…').element().parentElement!).backgroundColor).toBe(tokenBackground('nm-bg'));
	});
});
