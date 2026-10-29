// Who else has the project open and who's typing in its chat. While the project page is open it
// sends a heartbeat (the server counts someone as here until their last one is 45 s old), asks
// who's here when someone arrives or leaves, and keeps typing names up for a few seconds after
// each ping. Everything comes over the project's live topic, through the page's ProjectDeploy.

import { expireTyping, shouldSend, typingNames, withPing, type TypingMap } from '$lib/presenceView';
import { projectApi } from '$lib/projectApi';
import type { ProjectDeploy } from '$lib/projectDeploy.svelte';

/** Comfortably inside the server's 45 s expiry, so a short network hiccup doesn't drop anyone. */
const HEARTBEAT_MS = 20_000;
/** A typing name stays up this long after its last ping. */
const TYPING_TTL_MS = 6_000;
/** The person's own pings go out at most this often while they type. */
const TYPING_PING_MS = 3_000;
const EXPIRY_CHECK_MS = 1_000;

export interface PresentPerson {
	user_id: string;
	name: string;
	avatar_url: string | null;
}

export class ProjectPresence {
	people = $state<PresentPerson[]>([]);
	me = $state<string | null>(null);
	#typing = $state<TypingMap>({});
	readonly projectId: string;
	#timers: ReturnType<typeof setInterval>[] = [];
	#unlisten: (() => void) | null = null;
	#lastPing: number | null = null;
	#onPageHide = () => this.#leave();

	constructor(projectId: string) {
		this.projectId = projectId;
	}

	/** The names of others typing now, the first to start first. */
	get typing(): string[] {
		return typingNames(this.#typing, this.me ?? '');
	}

	start(deploy: ProjectDeploy) {
		void this.#beat().then(() => this.refresh());
		this.#timers.push(setInterval(() => void this.#beat(), HEARTBEAT_MS));
		this.#timers.push(
			setInterval(() => {
				const fresh = expireTyping(this.#typing, Date.now(), TYPING_TTL_MS);
				if (Object.keys(fresh).length !== Object.keys(this.#typing).length) this.#typing = fresh;
			}, EXPIRY_CHECK_MS),
		);
		this.#unlisten = deploy.onEvent((data) => this.#handle(data));
		window.addEventListener('pagehide', this.#onPageHide);
	}

	async refresh() {
		try {
			const presence = await projectApi<{ me: string; people: PresentPerson[] }>(this.projectId, 'presence');
			this.me = presence.me;
			this.people = presence.people;
		} catch {
			// Who's here is a nicety: the next arrival or heartbeat tries again.
		}
	}

	/** The person is typing in the project's chat: tell the others, at most every few seconds. */
	ping() {
		const now = Date.now();
		if (!shouldSend(this.#lastPing, now, TYPING_PING_MS)) return;
		this.#lastPing = now;
		void projectApi(this.projectId, 'typing', { method: 'POST' }).catch(() => null);
	}

	stop() {
		for (const timer of this.#timers) clearInterval(timer);
		this.#timers = [];
		this.#unlisten?.();
		this.#unlisten = null;
		window.removeEventListener('pagehide', this.#onPageHide);
		this.#leave();
	}

	async #beat() {
		await projectApi(this.projectId, 'presence', { method: 'POST' }).catch(() => null);
	}

	/** Says goodbye even while the page is unloading (keepalive); the expiry covers it when it can't. */
	#leave() {
		void fetch(`/projects/${this.projectId}/api/presence`, { method: 'DELETE', keepalive: true }).catch(() => null);
	}

	#handle(data: string) {
		let event: { kind?: string; user_id?: string; name?: string };
		try {
			event = JSON.parse(data);
		} catch {
			return;
		}
		if (event.kind === 'PresenceChanged') void this.refresh();
		if (event.kind === 'Typing' && event.user_id && event.user_id !== this.me) {
			this.#typing = withPing(this.#typing, event.user_id, event.name ?? '', Date.now());
		}
	}
}
