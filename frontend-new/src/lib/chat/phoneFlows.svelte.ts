import type { ClarifyProgress } from '$lib/ds/thread/PhoneClarifySheet.svelte';

// What the phone's chat flows keep while a chat is open (UX contract phone-chat-flows): the
// questions the person is part-way through (by the block's key: closing the sheet parks them, and
// Answer brings back the same question with the earlier answers; leaving the chat drops them, G9),
// what was answered here and how many were defaults, the plans the thread shows (so the run bar
// knows one waits), plans run from here, the composer's note while the plan is being changed, and
// the crew sessions running in the chat. ChatView makes one per chat and hands it to the blocks.

/** A plan the thread shows: its newest version, where it stands, its steps and who owns them. */
export type PlanSeen = { version: number; state: string; steps: number; owners: string[] };

export class PhoneFlows {
	/** Questions part-way, by the block's key. */
	progress = $state<Record<string, ClarifyProgress>>({});
	/** The questions on screen (or parked), by the block's key. */
	asking = $state<string | null>(null);
	sheetOpen = $state(false);
	/** Questions whose sheet opened on its own already: closing them parks them rather than reopening. */
	opened = new Set<string>();
	/** Answered on this visit: how many questions, and how many took the recommendation. */
	answered = $state<Record<string, { count: number; defaults: number }>>({});
	/** Plans the thread shows, by the message carrying them. */
	plans = $state<Record<string, PlanSeen>>({});
	/** Plans sent to run (or to research) from here: their bar goes. */
	ran = $state<Record<string, true>>({});
	runOpen = $state(false);
	/** "Changing the plan · v1" over the composer, until the next message goes. */
	notice = $state<string | null>(null);
	/** Crew sessions running in this chat, by id: who. */
	sessions = $state<Record<string, string>>({});

	/** Who's working, in the order they started. */
	get working(): string[] {
		return Object.values(this.sessions);
	}

	sessionStarted(id: string, name: string) {
		this.sessions = { ...this.sessions, [id]: name };
	}

	sessionEnded(id: string) {
		const { [id]: _gone, ...rest } = this.sessions;
		this.sessions = rest;
	}

	/** Another chat opened: nothing carries over. */
	reset() {
		this.progress = {};
		this.asking = null;
		this.sheetOpen = false;
		this.opened.clear();
		this.answered = {};
		this.plans = {};
		this.ran = {};
		this.runOpen = false;
		this.notice = null;
		this.sessions = {};
	}
}
