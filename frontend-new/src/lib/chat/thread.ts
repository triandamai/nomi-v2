import type { RenderedMessage } from '$lib/adapters/api';
import { dayKey, dayLabel } from './dates';
import { failureOf, type Failure } from './failure';
import { groupReasoning } from './reasoning';

// The thread as it reads, from the chat's messages: a divider at each new day, where the person
// left off (the NEW divider), each message of theirs, and each reply with the thinking that led
// to it, whether it failed, and the follow-ups the crew offered after it. The latest turn (no
// message of the person's after it) is the one that can be edited or asked for again.

export type ThreadEntry =
	| { type: 'day'; key: string; label: string }
	| { type: 'new'; count: number }
	| { type: 'user'; message: RenderedMessage; latest: boolean }
	| {
			type: 'reply';
			message: RenderedMessage;
			reasoning: string[];
			thinkingOnly: boolean;
			latest: boolean;
			/** When the person asked what this answers. */
			turnStart: string | null;
			failure: Failure | null;
			/** Next questions the crew offered after it, while they still apply. */
			followUps: string[];
	  };

type Options = { locale: string; now: Date; timeZone?: string; newFromId: string | null };

const suggestionsOf = (message: RenderedMessage): string[] | null => {
	const blocks = message.content_blocks ?? [];
	if (blocks.length === 0 || !blocks.every((block) => block.kind === 'suggestions')) return null;
	return blocks.flatMap((block) => {
		const items: unknown = (block as { items?: unknown }).items;
		return Array.isArray(items) ? items.filter((item): item is string => typeof item === 'string') : [];
	});
};

export function threadEntries(messages: RenderedMessage[], { locale, now, timeZone, newFromId }: Options): ThreadEntry[] {
	const items = groupReasoning(messages);
	const lastUser = items.findLastIndex((item) => item.message.sender === 'user');
	const newAt = newFromId === null ? -1 : items.findIndex((item) => item.message.id === newFromId);
	const entries: ThreadEntry[] = [];
	let day = '';
	let turnStart: string | null = null;

	items.forEach((item, index) => {
		const { message } = item;
		const key = dayKey(message.created_at, timeZone);
		if (key !== day) {
			day = key;
			entries.push({ type: 'day', key, label: dayLabel(message.created_at, locale, now, timeZone) });
		}
		if (index === newAt) entries.push({ type: 'new', count: items.length - newAt });
		const latest = index >= lastUser;
		if (message.sender === 'user') {
			turnStart = message.created_at;
			entries.push({ type: 'user', message, latest });
			return;
		}
		const offered = suggestionsOf(message);
		if (offered) {
			const before = entries.findLast((entry) => entry.type === 'reply');
			if (before?.type === 'reply') before.followUps = latest ? offered : [];
			return;
		}
		entries.push({ type: 'reply', message, reasoning: item.reasoning, thinkingOnly: item.thinkingOnly, latest, turnStart, failure: failureOf(message), followUps: [] });
	});
	return entries;
}

/** How long a reply took after the person asked: "9s", "2m 5s". */
export function turnDuration(askedAt: string | null, repliedAt: string): string | undefined {
	if (!askedAt) return undefined;
	const seconds = Math.max(0, Math.floor((Date.parse(repliedAt) - Date.parse(askedAt)) / 1000));
	return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}
