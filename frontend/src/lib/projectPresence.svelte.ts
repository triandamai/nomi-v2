// Who else has the project open and who's typing in its chat. While the project page is open its
// live socket (`/projects/{id}/ws`, owned by ProjectDeploy) keeps the person counted as here,
// carries their whereabouts, cursor and typing pings, and hears when someone arrives or leaves.

import { expireTyping, shouldSend, typingNames, withPing, type TypingMap } from '$lib/presenceView';
import { projectApi } from '$lib/projectApi';
import type { ProjectDeploy } from '$lib/projectDeploy.svelte';

/** A typing name stays up this long after its last ping. */
const TYPING_TTL_MS = 6_000;
/** The person's own pings go out at most this often while they type. */
const TYPING_PING_MS = 3_000;
const EXPIRY_CHECK_MS = 1_000;

export interface PresentPerson {
	user_id: string;
	name: string;
	avatar_url: string | null;
	/** What they're looking at: preview, code or terminal. */
	view?: 'preview' | 'code' | 'terminal' | null;
	/** The file open in their code view. */
	path?: string | null;
}

/** Where someone's pointer is over a view, as fractions of it, and when it last moved. */
export interface PeerPointer {
	name: string;
	x: number;
	y: number;
	view: string;
	at: number;
}

export type Whereabouts = { view: 'preview' | 'code' | 'terminal'; path?: string | null };

/** A pointer that hasn't moved for this long is no longer shown. */
const POINTER_TTL_MS = 5_000;
/** The person's own pointer goes out at most this often. */
const POINTER_MS = 120;

export class ProjectPresence {
	people = $state<PresentPerson[]>([]);
	/** Others' pointers, by user id. */
	pointers = $state<Record<string, PeerPointer>>({});
	#where: Whereabouts | null = null;
	#lastPointer = 0;
	me = $state<string | null>(null);
	#typing = $state<TypingMap>({});
	readonly projectId: string;
	#deploy: ProjectDeploy | null = null;
	#refreshing: Promise<void> | null = null;
	#timers: ReturnType<typeof setInterval>[] = [];
	#unlisten: (() => void) | null = null;
	#lastPing: number | null = null;

	constructor(projectId: string) {
		this.projectId = projectId;
	}

	/** The names of others typing now, the first to start first. */
	get typing(): string[] {
		return typingNames(this.#typing, this.me ?? '');
	}

	start(deploy: ProjectDeploy) {
		this.#deploy = deploy;
		if (this.#where) {
			deploy.send({ op: 'where', view: this.#where.view, path: this.#where.path ?? null });
		}
		void this.refresh();
		this.#timers.push(
			setInterval(() => {
				const fresh = expireTyping(this.#typing, Date.now(), TYPING_TTL_MS);
				if (Object.keys(fresh).length !== Object.keys(this.#typing).length) this.#typing = fresh;
			}, EXPIRY_CHECK_MS),
		);
		this.#unlisten = deploy.onEvent((data) => this.#handle(data));
	}

	async refresh() {
		if (this.#refreshing) return this.#refreshing;
		this.#refreshing = (async () => {
			try {
				const presence = await projectApi<{ me: string; people: PresentPerson[] }>(this.projectId, 'presence');
				this.me = presence.me;
				this.people = presence.people;
			} catch {
				// Who's here is a nicety: the next arrival tries again.
			} finally {
				this.#refreshing = null;
			}
		})();
		return this.#refreshing;
	}

	/** Where the person is now: told to the others straight away over the project socket. */
	moveTo(where: Whereabouts) {
		if (this.#where && this.#where.view === where.view && (this.#where.path ?? null) === (where.path ?? null)) return;
		this.#where = where;
		if (this.#deploy) {
			this.#deploy.send({ op: 'where', view: where.view, path: where.path ?? null });
		} else {
			void projectApi(this.projectId, 'presence', { method: 'POST', body: where }).catch(() => null);
		}
	}

	/** The person's pointer over a view, as fractions of it (sent a few times a second at most when others are here). */
	point(view: string, x: number, y: number) {
		if (this.people.length === 0) return;
		const now = Date.now();
		if (now - this.#lastPointer < POINTER_MS) return;
		this.#lastPointer = now;
		const at = { view, x: Math.min(1, Math.max(0, x)), y: Math.min(1, Math.max(0, y)) };
		if (this.#deploy) {
			this.#deploy.send({ op: 'cursor', ...at });
		} else {
			void projectApi(this.projectId, 'cursor', { method: 'POST', body: at }).catch(() => null);
		}
	}

	/** Pointers still moving, in one view. */
	pointersIn(view: string, now = Date.now()): (PeerPointer & { user_id: string })[] {
		return Object.entries(this.pointers)
			.filter(([, pointer]) => pointer.view === view && now - pointer.at < POINTER_TTL_MS)
			.map(([user_id, pointer]) => ({ user_id, ...pointer }));
	}

	/** The person is typing in the project's chat: tell the others, at most every few seconds. */
	ping() {
		const now = Date.now();
		if (!shouldSend(this.#lastPing, now, TYPING_PING_MS)) return;
		this.#lastPing = now;
		if (this.#deploy) {
			this.#deploy.send({ op: 'typing' });
		} else {
			void projectApi(this.projectId, 'typing', { method: 'POST' }).catch(() => null);
		}
	}

	stop() {
		for (const timer of this.#timers) clearInterval(timer);
		this.#timers = [];
		this.#unlisten?.();
		this.#unlisten = null;
		this.#deploy = null;
	}

	#handle(data: string) {
		let event: { kind?: string; user_id?: string; name?: string; x?: number; y?: number; view?: string };
		try {
			event = JSON.parse(data);
		} catch {
			return;
		}
		if (event.kind === 'PresenceChanged' && (!this.me || event.user_id !== this.me)) void this.refresh();
		if (event.kind === 'Cursor' && event.user_id && event.user_id !== this.me && typeof event.x === 'number' && typeof event.y === 'number') {
			this.pointers = { ...this.pointers, [event.user_id]: { name: event.name ?? '', x: event.x, y: event.y, view: event.view ?? 'preview', at: Date.now() } };
		}
		if (event.kind === 'Typing' && event.user_id && event.user_id !== this.me) {
			this.#typing = withPing(this.#typing, event.user_id, event.name ?? '', Date.now());
		}
	}
}
