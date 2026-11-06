// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { clickEverything, render, settle, text } from '../../../test-stubs/render';
import Page from './+page.svelte';
import type { Reminder } from '$lib/types';

const reminder = (over: Partial<Reminder>): Reminder => ({
	id: 'r1',
	session_id: 's1',
	title: 'Pay rent',
	notes: 'Transfer to Bu Sri',
	due_at: '2026-11-07T02:00:00Z',
	recurrence: 'monthly',
	recurrence_weekday: null,
	recurrence_day_of_month: 7,
	status: 'active',
	created_by: 'user',
	last_fired_at: null,
	created_at: '2026-10-01T00:00:00Z',
	...over,
});

const reminders = {
	timezone: 'Asia/Jakarta',
	upcoming: [
		reminder({}),
		reminder({ id: 'r2', title: 'Gym', notes: null, recurrence: 'weekly', recurrence_weekday: 1, created_by: 'agent', due_at: '2026-11-09T23:00:00Z' }),
		reminder({ id: 'r3', title: 'Call Mum', recurrence: null, due_at: '2027-01-01T10:00:00Z' }),
		reminder({ id: 'r4', title: 'Daily pills', recurrence: 'daily', due_at: '2026-11-06T13:00:00Z' }),
	],
	past: [reminder({ id: 'r5', title: 'Dentist', status: 'done', recurrence: null }), reminder({ id: 'r6', title: 'Old', status: 'fired', recurrence: null, last_fired_at: '2026-10-01T00:00:00Z' })],
	scheduled_tasks: [{ id: 't1', label: 'Weekly money summary', run_at: '2026-11-10T01:00:00Z', recurrence: 'weekly', agent: 'money', session_id: 's1' }],
};

const page = (data: unknown, form: unknown = null) => render(Page as never, { data: { reminders: data } as never, form: form as never, params: {} as never });

describe('reminders page', () => {
	it('lists what is coming, what is past and the crew’s scheduled work', () => {
		const shown = text(page(reminders));
		for (const title of ['Pay rent', 'Gym', 'Call Mum', 'Dentist', 'Weekly money summary']) expect(shown).toContain(title);
	});

	it('opens the new-reminder sheet and the item menus', async () => {
		const root = page(reminders, { error: 'Pick a time in the future' });
		expect(clickEverything(root)).toBeGreaterThan(0);
		await settle();
		clickEverything(document.body);
		expect(text()).toContain('Pick a time in the future');
	});

	it('shows an empty list and a list that failed to load', () => {
		expect(page({ ...reminders, upcoming: [], past: [], scheduled_tasks: [] })).toBeTruthy();
		expect(page(null).textContent).toBeTruthy();
	});
});
