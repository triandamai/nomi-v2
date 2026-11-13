import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FakeSocket } from '$lib/live/fakeSocket';
import { connectChat, parseChatEvent } from './events';

describe('parseChatEvent', () => {
	it('reads the envelopes the chat relay sends', () => {
		const turn = '7d1f6b0e-0000-4000-8000-000000000001';
		expect(parseChatEvent(JSON.stringify({ kind: 'Delta', turn_job_id: turn, event: { TextDelta: { index: 0, text: 'Hi' } } }))).toEqual({
			kind: 'Delta',
			turn_job_id: turn,
			event: { TextDelta: { index: 0, text: 'Hi' } },
		});
		expect(parseChatEvent('{"kind":"TurnFailed","turn_job_id":"t","error":"DeepSeek is having trouble right now."}')).toMatchObject({ kind: 'TurnFailed', error: 'DeepSeek is having trouble right now.' });
		expect(parseChatEvent('{"kind":"MessagesDeleted","message_ids":["a","b"]}')).toEqual({ kind: 'MessagesDeleted', message_ids: ['a', 'b'] });
		expect(parseChatEvent('{"kind":"SessionDeleted"}')).toEqual({ kind: 'SessionDeleted' });
		expect(parseChatEvent('{"kind":"ChatSettled","outcome":"done","waiting":null,"agent":"Dana"}')).toMatchObject({ kind: 'ChatSettled', outcome: 'done' });
	});

	it('drops anything it can’t read or doesn’t know', () => {
		expect(parseChatEvent('not json')).toBeNull();
		expect(parseChatEvent('{"kind":"SomethingNew"}')).toBeNull();
		expect(parseChatEvent('{"kind":"TurnCompleted"}')).toBeNull();
		expect(parseChatEvent('[1]')).toBeNull();
		expect(parseChatEvent('{"kind":"SessionRenamed","title":5}')).toBeNull();
		expect(parseChatEvent('{"kind":"MessagesDeleted","message_ids":"a"}')).toBeNull();
	});
});

describe('connectChat', () => {
	beforeEach(() => vi.useFakeTimers());
	afterEach(() => vi.useRealTimers());

	it("follows one chat's feed, says when it (re)opened, and hands over what it hears", () => {
		const sockets: { url: string; socket: FakeSocket }[] = [];
		const opened = vi.fn();
		const event = vi.fn();
		const stop = connectChat({
			sessionId: 'abc',
			handlers: { opened, event, lost: vi.fn() },
			openSocket: (url) => {
				const socket = new FakeSocket();
				sockets.push({ url, socket });
				return socket;
			},
		});
		expect(sockets[0].url).toBe('/chat/abc/ws');
		sockets[0].socket.fire('open');
		expect(opened).toHaveBeenLastCalledWith(false);
		sockets[0].socket.fire('message', { data: '{"kind":"SessionRenamed","title":"Rent"}' });
		sockets[0].socket.fire('message', { data: 'garbage' });
		expect(event).toHaveBeenCalledTimes(1);
		expect(event).toHaveBeenCalledWith({ kind: 'SessionRenamed', title: 'Rent' });

		sockets[0].socket.fire('close', { code: 1006 });
		vi.advanceTimersByTime(2000);
		sockets[1].socket.fire('open');
		expect(opened).toHaveBeenLastCalledWith(true);

		stop();
		expect(sockets[1].socket.closed).toBe(true);
	});

	it('gives up when the server says the chat is gone or the sign-in is over', () => {
		const socket = new FakeSocket();
		const lost = vi.fn();
		connectChat({ sessionId: 'abc', handlers: { opened: vi.fn(), event: vi.fn(), lost }, openSocket: () => socket });
		socket.fire('close', { code: 4404 });
		expect(lost).toHaveBeenCalledWith(4404);
	});
});
