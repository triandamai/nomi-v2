import { describe, expect, it } from 'vitest';
import { answeredRequests, askLine, movableAsk, myOpenRequest, openRequests, requestMeta, type PublishRequestRow, type QueuedAsk } from './asks';

const ask = (over: Partial<QueuedAsk>): QueuedAsk => ({ id: 'a', text: 'Make the Pre-order button green', who: 'Mario Prasetyo', mine: true, state: 'running', created_at: '2026-11-14T02:14:00Z', ...over });
const request = (over: Partial<PublishRequestRow>): PublishRequestRow => ({
	id: 'r1',
	environment: 'production',
	note: null,
	status: 'pending',
	requested_by: 'u-sari',
	requested_by_name: 'Sari Wulandari',
	decided_by_name: null,
	decided_at: null,
	publish_id: null,
	created_at: '2026-11-14T02:21:00Z',
	...over,
});

describe('asks in line (Invite & build together, step 9)', () => {
	it('gives each ask who asked it and where it is, in the person’s own words', () => {
		const line = askLine([
			ask({ id: 'a1' }),
			ask({ id: 'a2', text: 'Add a Makanan section with nasi uduk\n\nSame pickup times.', who: 'Sari Wulandari', mine: false, state: 'next' }),
			ask({ id: 'a3', text: '', who: 'Sari Wulandari', mine: false, state: 'queued' }),
		]);
		expect(line).toEqual([
			{ text: 'Make the Pre-order button green', who: { name: 'Mario Prasetyo', initials: 'MP' }, state: 'running', meta: 'Mario Prasetyo · running' },
			{ text: 'Add a Makanan section with nasi uduk', who: { name: 'Sari Wulandari', initials: 'SW', peer: expect.any(Number) }, state: 'next', meta: 'Sari Wulandari · next' },
			{ text: '…', who: { name: 'Sari Wulandari', initials: 'SW', peer: expect.any(Number) }, state: 'queued', meta: 'Sari Wulandari · waiting' },
		]);
	});

	it('lets a teammate’s ask go first only when one of yours is ahead of it and it isn’t next already', () => {
		// The pen's two asks: Mario's running, Sari's next. Nothing to move: a running ask can't wait.
		expect(movableAsk([ask({ state: 'running' }), ask({ mine: false, state: 'next' })])).toBe(-1);
		// Mario's waits next, Sari's behind it: she can go first.
		expect(movableAsk([ask({ mine: false, state: 'running' }), ask({ state: 'next' }), ask({ mine: false, state: 'queued' }), ask({ mine: false, state: 'queued' })])).toBe(2);
		// Two of hers in line, none of his waiting: not his to reorder.
		expect(movableAsk([ask({ state: 'running' }), ask({ mine: false, state: 'next' }), ask({ mine: false, state: 'queued' })])).toBe(-1);
		// His own asks only: nobody to let go first.
		expect(movableAsk([ask({ state: 'next' }), ask({ state: 'queued' })])).toBe(-1);
	});
});

describe('asking to publish (Invite & build together, step 10)', () => {
	it('keeps the open requests, and the person’s own', () => {
		const rows = [request({ id: 'r1' }), request({ id: 'r2', status: 'declined' }), request({ id: 'r3', requested_by: 'u-me', requested_by_name: 'Me' })];
		expect(openRequests(rows, 'u-me').map((one) => one.id)).toEqual(['r1']);
		expect(myOpenRequest(rows, 'u-me')?.id).toBe('r3');
		expect(myOpenRequest(rows, null)).toBeNull();
	});

	it('says what a request holds: its note, the files that would go live, and when it was asked', () => {
		expect(requestMeta(request({ note: 'Makanan section' }), 2, 'UTC')).toBe('Makanan section · 2 files · asked 02:21');
		expect(requestMeta(request({}), 1, 'UTC')).toBe('1 file · asked 02:21');
		expect(requestMeta(request({}), null, 'UTC')).toBe('asked 02:21');
	});

	it('tells the person who asked how their request was answered, once', () => {
		const before = [request({ id: 'r1', requested_by: 'u-me' })];
		expect(answeredRequests(before, [request({ id: 'r1', requested_by: 'u-me', status: 'approved', decided_by_name: 'Mario' })], 'u-me')).toEqual([{ status: 'approved', by: 'Mario' }]);
		expect(answeredRequests(before, [request({ id: 'r1', requested_by: 'u-me', status: 'declined', decided_by_name: 'Mario' })], 'u-me')).toEqual([{ status: 'declined', by: 'Mario' }]);
		// Taken back by the person themselves, still open, or someone else's: nothing to say.
		expect(answeredRequests(before, [request({ id: 'r1', requested_by: 'u-me', status: 'cancelled' })], 'u-me')).toEqual([]);
		expect(answeredRequests(before, before, 'u-me')).toEqual([]);
		expect(answeredRequests([request({ id: 'r1' })], [request({ id: 'r1', status: 'approved' })], 'u-me')).toEqual([]);
	});
});
