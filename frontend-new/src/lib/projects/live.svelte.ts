import { createLiveLink, type LinkSocket, type LiveLink } from '$lib/live/link';
import type { PresentPerson } from '$lib/server/workspace';
import type { PeerPointer } from './peers';

// A project's live link (`/projects/{id}/ws`, relayed by ws-proxy) while its workspace is open
// (Flow / Build — Invite & build together TnwQT). Being connected keeps the person counted as here;
// the link carries where they are and their pointer, and hears when someone arrives, moves or
// leaves (then who's here is read again: the events don't say) and where others point.

export type Whereabouts = { view: 'preview' | 'code' | 'terminal'; path?: string | null };

/** The person's own pointer goes out at most this often. */
const POINTER_MS = 120;

export class ProjectLive {
	/** Everyone else with the project open, longest-present first. */
	people = $state<PresentPerson[]>([]);
	me = $state<string | null>(null);
	/** Others' pointers, by user id. */
	pointers = $state<Record<string, PeerPointer>>({});

	#id: string;
	#openSocket: (url: string) => LinkSocket;
	#link: LiveLink | null = null;
	#where: Whereabouts | null = null;
	#lastPointer = 0;
	#reading: Promise<void> | null = null;
	#again = false;

	constructor(projectId: string, openSocket: (url: string) => LinkSocket = (url) => new WebSocket(url)) {
		this.#id = projectId;
		this.#openSocket = openSocket;
	}

	start() {
		if (this.#link) return;
		this.#link = createLiveLink(
			{
				opened: () => {
					void this.refresh();
					this.#sendWhere();
				},
				message: (data) => this.#handle(String(data)),
				lost: () => {},
			},
			() => this.#openSocket(`/projects/${encodeURIComponent(this.#id)}/ws`),
		);
	}

	stop() {
		this.#link?.close();
		this.#link = null;
	}

	/** Reads who's here again: one read at a time, and once more when asked during one. */
	refresh(): Promise<void> {
		if (this.#reading) {
			this.#again = true;
			return this.#reading;
		}
		this.#reading = (async () => {
			do {
				this.#again = false;
				try {
					const response = await fetch(`/projects/${encodeURIComponent(this.#id)}/presence`);
					if (response.ok) {
						const presence = (await response.json()) as { me: string; people: PresentPerson[] };
						this.me = presence.me;
						this.people = presence.people;
					}
				} catch {
					// Who's here is a nicety: the next change reads it again.
				}
			} while (this.#again);
			this.#reading = null;
		})();
		return this.#reading;
	}

	/** Where the person is now, told to the others when it changes. */
	moveTo(where: Whereabouts) {
		const path = where.path ?? null;
		if (this.#where && this.#where.view === where.view && (this.#where.path ?? null) === path) return;
		this.#where = { view: where.view, path };
		this.#sendWhere();
	}

	/** The person's pointer over a view, as fractions of it: sent a few times a second at most, and only when someone else is here. */
	point(view: string, x: number, y: number) {
		if (!this.people.length) return;
		const now = Date.now();
		if (now - this.#lastPointer < POINTER_MS) return;
		this.#lastPointer = now;
		const clamp = (value: number) => Math.min(1, Math.max(0, value));
		this.#link?.send(JSON.stringify({ op: 'cursor', view, x: clamp(x), y: clamp(y) }));
	}

	#sendWhere() {
		if (this.#where) this.#link?.send(JSON.stringify({ op: 'where', view: this.#where.view, path: this.#where.path ?? null }));
	}

	#handle(data: string) {
		let event: { kind?: string; user_id?: string; name?: string; x?: number; y?: number; view?: string };
		try {
			event = JSON.parse(data);
		} catch {
			return;
		}
		if (event.kind === 'PresenceChanged' && event.user_id !== this.me) void this.refresh();
		if (event.kind === 'Cursor' && event.user_id && event.user_id !== this.me && typeof event.x === 'number' && typeof event.y === 'number') {
			this.pointers = { ...this.pointers, [event.user_id]: { name: event.name ?? '', x: event.x, y: event.y, view: event.view ?? 'preview', at: Date.now() } };
		}
	}
}
