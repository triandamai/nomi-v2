import type { HomeSummary } from '$lib/adapters/api';

// What waits on the person in their chats, from Home's summary (`needs_you` items: an approval a
// crew member paused on), and which of them are new since the app last looked: on a phone, a new
// one in another chat is said with a toast (Flow / Mobile Chat — Approvals & background, step 5,
// the interim for the lock screen: a phone's browser gets no push, UX contract D2).

export type NeedsYou = { key: string; sessionId: string; agent: string; at: string };

export function needsYou(summary: HomeSummary | null): NeedsYou[] {
	return (summary?.while_you_were_out ?? []).flatMap((item) =>
		item.kind === 'needs_you' && item.session_id ? [{ key: `${item.session_id}@${item.at}`, sessionId: item.session_id, agent: item.agent, at: item.at }] : [],
	);
}

/** The items not seen before, outside the chat the person has open. */
export function newNeedsYou(seen: ReadonlySet<string>, items: NeedsYou[], openChat: string | null): NeedsYou[] {
	return items.filter((item) => !seen.has(item.key) && item.sessionId !== openChat);
}
