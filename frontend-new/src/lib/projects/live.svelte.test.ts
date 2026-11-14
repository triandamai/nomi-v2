import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FakeSocket } from '$lib/live/fakeSocket';
import { ProjectLive } from './live.svelte';

const json = (body: unknown) => new Response(JSON.stringify(body));
let here: { me: string; people: { user_id: string; name: string; avatar_url: null; view: string | null; path: string | null }[] };
let reads: number;

beforeEach(() => {
	reads = 0;
	here = { me: 'u-me', people: [] };
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string) => {
			if (String(url) === '/projects/p1/presence') {
				reads += 1;
				return json(here);
			}
			return new Response(null, { status: 404 });
		}),
	);
});
afterEach(() => vi.unstubAllGlobals());

function opened() {
	const sockets: FakeSocket[] = [];
	const live = new ProjectLive('p1', (url) => {
		expect(url).toBe('/projects/p1/ws');
		const socket = new FakeSocket();
		sockets.push(socket);
		return socket;
	});
	live.start();
	sockets[0].fire('open');
	return { live, socket: sockets[0] };
}

describe('a project’s live link', () => {
	it('reads who’s here on opening, and again when someone arrives, moves or leaves', async () => {
		const { live, socket } = opened();
		await vi.waitFor(() => expect(live.me).toBe('u-me'));
		expect(live.people).toEqual([]);
		here = { me: 'u-me', people: [{ user_id: 'u-sari', name: 'Sari Wulandari', avatar_url: null, view: 'preview', path: null }] };
		socket.fire('message', { data: JSON.stringify({ kind: 'PresenceChanged', user_id: 'u-sari' }) });
		await vi.waitFor(() => expect(live.people.map((person) => person.name)).toEqual(['Sari Wulandari']));
		// Their own arrival isn't news to them.
		const before = reads;
		socket.fire('message', { data: JSON.stringify({ kind: 'PresenceChanged', user_id: 'u-me' }) });
		expect(reads).toBe(before);
		live.stop();
		expect(socket.closed).toBe(true);
	});

	it('tells the others where the person is, once per move, and again after reconnecting', async () => {
		const { live, socket } = opened();
		live.moveTo({ view: 'code', path: 'index.html' });
		live.moveTo({ view: 'code', path: 'index.html' });
		live.moveTo({ view: 'preview' });
		expect(socket.sent.map((one) => JSON.parse(one))).toEqual([
			{ op: 'where', view: 'code', path: 'index.html' },
			{ op: 'where', view: 'preview', path: null },
		]);
		live.stop();
	});

	it('shows others’ pointers, not their own, and sends theirs only while someone else is here', async () => {
		vi.useFakeTimers({ toFake: ['Date'] });
		vi.setSystemTime(10_000);
		const { live, socket } = opened();
		live.point('preview', 0.5, 0.5);
		expect(socket.sent).toEqual([]);
		here = { me: 'u-me', people: [{ user_id: 'u-sari', name: 'Sari', avatar_url: null, view: 'preview', path: null }] };
		socket.fire('message', { data: JSON.stringify({ kind: 'PresenceChanged', user_id: 'u-sari' }) });
		await vi.waitFor(() => expect(live.people).toHaveLength(1));
		live.point('preview', 1.4, -0.2);
		live.point('preview', 0.3, 0.3);
		expect(socket.sent.map((one) => JSON.parse(one))).toEqual([{ op: 'cursor', view: 'preview', x: 1, y: 0 }]);
		const at = Date.now();
		socket.fire('message', { data: JSON.stringify({ kind: 'Cursor', user_id: 'u-sari', name: 'Sari', x: 0.2, y: 0.4, view: 'preview' }) });
		socket.fire('message', { data: JSON.stringify({ kind: 'Cursor', user_id: 'u-me', name: 'Me', x: 0.9, y: 0.9, view: 'preview' }) });
		socket.fire('message', { data: 'not json' });
		expect(live.pointers).toEqual({ 'u-sari': { name: 'Sari', x: 0.2, y: 0.4, view: 'preview', at } });
		live.stop();
		vi.useRealTimers();
	});
});
