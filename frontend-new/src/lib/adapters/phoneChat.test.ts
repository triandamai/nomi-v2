import { describe, expect, it } from 'vitest';
import { addedSteps, approvalPreview, biggerModel, countdown, finishedIn, formDate, isPastDate, isReminderForm, ownerList, phoneFailureView, previewOrder, recipientOf, resetsOn, sendsEmail, sendsSomething } from './phoneChat';

const models = [
	{ key: 'conn:a:flash', label: 'deepseek-flash', context: 64_000 },
	{ key: 'admin:1', label: 'nomi-large', context: 200_000 },
	{ key: 'admin:2', label: 'nomi-mid', context: 128_000 },
];

describe('phoneFailureView (UX contract S4)', () => {
	it('out of credits: Nomi credits first where they can be used, then Retry', () => {
		expect(phoneFailureView('no_credits', { retry: true, canUseCredits: true })).toEqual({
			title: 'Out of credits',
			actions: [
				{ action: 'credits', label: 'Switch to Nomi credits' },
				{ action: 'retry', label: 'Retry' },
			],
		});
		expect(phoneFailureView('no_credits', { retry: false, canUseCredits: false }).actions).toEqual([{ action: 'models', label: 'Choose a model' }]);
	});

	it('rate limited: Retry and Try another model', () => {
		expect(phoneFailureView('rate_limited', { retry: true, canUseCredits: true }).actions.map((one) => one.label)).toEqual(['Retry', 'Try another model']);
	});

	it('chat too long: a new chat, or the smallest model that reads more', () => {
		const view = phoneFailureView('too_long', { retry: false, canUseCredits: true, models, current: 'conn:a:flash' });
		expect(view.title).toBe('Chat too long');
		expect(view.actions).toEqual([
			{ action: 'new_chat', label: 'Start a new chat' },
			{ action: 'switch', model: 'admin:2', label: 'Use a 128K model' },
		]);
		expect(phoneFailureView('too_long', { retry: false, canUseCredits: true, models, current: 'admin:1' }).actions[1]).toEqual({ action: 'models', label: 'Choose a model' });
	});

	it('the other kinds, as the contract names them', () => {
		expect(phoneFailureView('bad_key', { retry: false, canUseCredits: true })).toEqual({
			title: 'Key rejected',
			actions: [
				{ action: 'own_key', label: 'Update key' },
				{ action: 'credits', label: 'Use Nomi credits' },
			],
		});
		expect(phoneFailureView('no_model', { retry: false, canUseCredits: true }).title).toBe('Model retired');
		expect(phoneFailureView('provider_down', { retry: true, canUseCredits: true, models, current: 'conn:a:flash' })).toEqual({
			title: 'Provider down',
			actions: [
				{ action: 'retry', label: 'Retry' },
				{ action: 'switch', model: 'admin:1', label: 'Try nomi-large' },
			],
		});
		expect(phoneFailureView('timed_out', { retry: true, canUseCredits: false }).actions.map((one) => one.label)).toEqual(['Retry', 'Try a faster model']);
		expect(phoneFailureView('quota', { retry: false, canUseCredits: true })).toEqual({
			title: 'Out of Nomi credits',
			actions: [
				{ action: 'upgrade', label: 'Upgrade plan' },
				{ action: 'own_key', label: 'Use my own key' },
			],
		});
		// The remaining kinds keep the desktop's title and moves.
		expect(phoneFailureView('other', { retry: true, canUseCredits: true })).toEqual({ title: 'Couldn’t reply', actions: [{ action: 'retry', label: 'Retry' }] });
		expect(phoneFailureView('unreachable', { retry: false, canUseCredits: true }).actions).toEqual([]);
	});
});

describe('the plan', () => {
	it('marks the steps a new version added, and none on the first', () => {
		expect(addedSteps(null, ['Find trains', 'Book stay'])).toEqual([false, false]);
		expect(addedSteps(['Find trains', 'Food spots'], ['Find trains', 'Rainy-day backup'])).toEqual([false, true]);
	});

	it('names each owner once', () => {
		expect(ownerList(['Nomi', 'Nomi', 'Dana', 'Tara'])).toBe('Nomi, Dana, Tara');
	});

	it('says how long it took', () => {
		expect(finishedIn('2026-11-17T07:00:00Z', '2026-11-17T07:03:40Z')).toEqual({ minutes: 3, seconds: 40 });
	});

	it('picks the smallest bigger model, or none', () => {
		expect(biggerModel(models, 'admin:1')).toBeNull();
		expect(biggerModel(models, null)?.key).toBe('conn:a:flash');
	});
});

describe('credits and time', () => {
	it('resets on the first of next month, in the person’s language', () => {
		expect(resetsOn(new Date('2026-11-17T10:00:00Z'), 'en', 'Asia/Jakarta')).toBe('1 Dec');
		expect(resetsOn(new Date('2026-12-31T20:00:00Z'), 'en', 'Asia/Jakarta')).toBe('1 Feb');
		expect(resetsOn(new Date('2026-11-17T10:00:00Z'), 'id', 'Asia/Jakarta')).toBe('1 Des');
	});

	it('counts down in minutes and seconds', () => {
		expect(countdown(24)).toBe('0:24');
		expect(countdown(75)).toBe('1:15');
	});

	it('knows a date that has passed, in the person’s day', () => {
		const now = new Date('2026-11-17T20:00:00Z');
		expect(isPastDate('2026-10-25', now, 'Asia/Jakarta')).toBe(true);
		expect(isPastDate('2026-11-18', now, 'Asia/Jakarta')).toBe(false);
		expect(isPastDate('not a date', now)).toBe(false);
	});
});

describe('approvals', () => {
	it('says Don’t send for a tool that sends, and Decline for the rest (D5)', () => {
		expect(sendsSomething('gmail.send')).toBe(true);
		expect(sendsSomething('whatsapp.message')).toBe(true);
		expect(sendsSomething('files.delete')).toBe(false);
		expect(sendsEmail('gmail.send')).toBe(true);
		expect(sendsEmail('calendar.create')).toBe(false);
	});

	it('reads the message out of the approval’s input', () => {
		const fields = [
			{ key: 'to', value: 'bu.sri@example.com' },
			{ key: 'subject', value: 'Sewa November' },
			{ key: 'body', value: 'Selamat siang Bu Sri' },
		];
		expect(recipientOf(fields)).toBe('bu.sri@example.com');
		expect(approvalPreview(fields)).toEqual({ to: 'bu.sri@example.com', subject: 'Sewa November', body: 'Selamat siang Bu Sri', attachment: undefined });
	});
});

describe('forms (gate 3 #9, answer 2)', () => {
	it('writes a date as the content rules do', () => {
		expect(formDate('2026-01-25', 'en')).toBe('25 Jan 2026');
		expect(formDate('2026-10-25', 'id')).toBe('25 Okt 2026');
		expect(formDate('09:00', 'en')).toBe('09:00');
	});

	it('knows a reminder from any other form', () => {
		expect(isReminderForm([{ key: 'date', label: 'Date', type: 'date' }, { key: 'repeat', label: 'Repeat', type: 'select' }])).toBe(true);
		expect(isReminderForm([{ key: 'date', label: 'Date', type: 'date' }, { key: 'guests', label: 'Guests', type: 'number' }])).toBe(false);
	});
});

describe('the approval preview (gate 3 #3)', () => {
	it('puts who it goes to first, then the subject', () => {
		expect(previewOrder([{ key: 'subject' }, { key: 'body' }, { key: 'to' }]).map((field) => field.key)).toEqual(['to', 'subject', 'body']);
	});
});
