import { m } from '$lib/paraglide/messages';
import { clock } from '$lib/chat/dates';
import { initials, peerFor } from './peers';

// Two people, one Koda (Flow / Build — Invite & build together TnwQT, steps 9–10): the asks waiting
// in the project's chat, and asking the project's admins to publish. Adapts the backend's queue
// (`GET /api/sessions/:id/queue`) and publish requests (`/api/projects/:id/deploy/requests`) to the
// design system's props.

/** One ask in the chat's line, as `turn_queue.rs` gives it. */
export type QueuedAsk = { id: string; text: string; who: string; mine: boolean; state: 'running' | 'next' | 'queued'; created_at: string };

/** A request to publish, as `publish_requests.rs` gives it. */
export type PublishRequestRow = {
	id: string;
	environment: string;
	note: string | null;
	status: 'pending' | 'approved' | 'declined' | 'cancelled';
	requested_by: string;
	requested_by_name: string;
	decided_by_name: string | null;
	decided_at: string | null;
	publish_id: string | null;
	created_at: string;
};

export type LineAsk = { text: string; who: { name: string; initials: string; peer?: 1 | 2 }; state: QueuedAsk['state']; meta: string };

const WHERE: Record<QueuedAsk['state'], (name: string) => string> = {
	running: (name) => m.queue_ask_running({ name }),
	next: (name) => m.queue_ask_next({ name }),
	queued: (name) => m.queue_ask_waiting({ name }),
};

/** The asks as the line shows them: the first line of each, who asked (in their colour unless it's yours) and where it is. */
export function askLine(asks: QueuedAsk[]): LineAsk[] {
	return asks.map((ask) => ({
		text: ask.text.split('\n').find((line) => line.trim())?.trim() ?? '…',
		who: { name: ask.who, initials: initials(ask.who), ...(!ask.mine && { peer: peerFor(ask.who) }) },
		state: ask.state,
		meta: WHERE[ask.state](ask.who),
	}));
}

/**
 * The teammate's ask the person can let go first, or -1: one that's waiting behind one of the
 * person's own and isn't next already. A running ask can't be moved, so with one running and one
 * next there's nothing to reorder.
 */
export function movableAsk(asks: QueuedAsk[]): number {
	let mineWaiting = false;
	for (const [index, ask] of asks.entries()) {
		if (ask.state === 'running') continue;
		if (ask.mine) mineWaiting = true;
		else if (mineWaiting && ask.state === 'queued') return index;
	}
	return -1;
}

/** Others' requests still waiting for an admin. */
export const openRequests = (rows: PublishRequestRow[], me: string | null) => rows.filter((row) => row.status === 'pending' && row.requested_by !== me);

/** The person's own request still waiting, if there is one. */
export const myOpenRequest = (rows: PublishRequestRow[], me: string | null) => (me ? (rows.find((row) => row.status === 'pending' && row.requested_by === me) ?? null) : null);

/** A request's meta: its note, how many files would go live, and when it was asked. */
export function requestMeta(row: PublishRequestRow, files: number | null, timeZone?: string): string {
	return [row.note, files ? (files === 1 ? m.publish_request_files_one() : m.publish_request_files({ count: files })) : null, m.publish_request_asked_at({ time: clock(row.created_at, timeZone) })]
		.filter(Boolean)
		.join(' · ');
}

/** The person's requests an admin answered since `before`: published or declined, and by whom. */
export function answeredRequests(before: PublishRequestRow[], after: PublishRequestRow[], me: string | null): { status: 'approved' | 'declined'; by: string }[] {
	const waiting = new Set(before.filter((row) => row.status === 'pending' && row.requested_by === me).map((row) => row.id));
	return after
		.filter((row) => waiting.has(row.id) && (row.status === 'approved' || row.status === 'declined'))
		.map((row) => ({ status: row.status as 'approved' | 'declined', by: row.decided_by_name ?? '' }));
}
