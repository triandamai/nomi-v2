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
			/** Action cards posted on their own earlier in the turn, drawn under this reply's text. */
			cards: RenderedMessage[];
			/** The text only repeats what a card already shows, so only the card is drawn. */
			quiet: boolean;
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

type Block = { kind: string; view?: unknown } & Record<string, unknown>;
type Reply = Extract<ThreadEntry, { type: 'reply' }>;

// The cards that ask the person to do something: answer, pick, fill in, approve, connect.
const ACTION_KINDS = new Set(['questions', 'approval_request', 'workspace_connect', 'connector_connect']);
const ACTION_VIEWS = new Set(['decision', 'form', 'selection', 'cart', 'boundary']);
// A card with fewer names than this can't be told apart from a sentence that happens to use one.
const MIN_LABELS = 2;

const blocksOf = (message: RenderedMessage): Block[] => (message.content_blocks ?? []) as Block[];
const isAction = (block: Block) => ACTION_KINDS.has(block.kind) || (block.kind === 'view' && ACTION_VIEWS.has(String(block.view)));
const hasAction = (message: RenderedMessage) => blocksOf(message).some(isAction);
const names = (value: unknown, field: string): string[] =>
	Array.isArray(value) ? value.map((item) => (item as Record<string, unknown>)?.[field]).filter((name): name is string => typeof name === 'string' && name.trim() !== '') : [];

/** The names an action card shows the person: its options, items or fields. */
function cardLabels(block: Block): string[] {
	if (block.kind === 'questions') return (Array.isArray(block.questions) ? block.questions : []).flatMap((question) => names((question as Block).options, 'label'));
	if (block.kind !== 'view') return [];
	switch (block.view) {
		case 'decision':
			return names(block.options, 'label');
		case 'selection':
		case 'cart':
			return names(block.items, 'name');
		case 'boundary':
			return names(block.alternatives, 'title');
		case 'form':
			return names(block.fields, 'label');
		default:
			return [];
	}
}

const plain = (text: string) => text.toLowerCase().replace(/\s+/g, ' ');

/** Whether the text only says again what one of the cards shows: it names every option on it. */
export function repeatsCard(text: string, cards: RenderedMessage[]): boolean {
	const said = plain(text);
	if (!said.trim()) return false;
	return cards.some((card) =>
		blocksOf(card)
			.filter(isAction)
			.some((block) => {
				const labels = cardLabels(block);
				return labels.length >= MIN_LABELS && labels.every((label) => said.includes(plain(label.trim())));
			}),
	);
}

// An action card the crew posted on its own (as a tool ran) moves under the reply that follows it
// from the same crew member in the same turn: the person reads what was said, then acts on it.
function foldCards(entries: ThreadEntry[]): ThreadEntry[] {
	const out: ThreadEntry[] = [];
	const held: Reply[] = [];
	const release = () => out.push(...held.splice(0));
	for (const entry of entries) {
		if (entry.type === 'user' || entry.type === 'day') release();
		if (entry.type !== 'reply') {
			out.push(entry);
			continue;
		}
		const { message } = entry;
		if (!entry.failure && !entry.thinkingOnly && hasAction(message)) {
			held.push(entry);
			continue;
		}
		const mine = held.filter((card) => card.message.agent_display_name === message.agent_display_name);
		if (mine.length && !entry.failure && !entry.thinkingOnly && message.content.trim()) {
			for (const card of mine) held.splice(held.indexOf(card), 1);
			entry.reasoning = [...mine.flatMap((card) => card.reasoning), ...entry.reasoning];
			entry.cards = mine.map((card) => card.message);
		}
		release();
		out.push(entry);
	}
	release();
	for (const entry of out) if (entry.type === 'reply') entry.quiet = repeatsCard(entry.message.content, [entry.message, ...entry.cards]);
	return out;
}

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
		entries.push({ type: 'reply', message, reasoning: item.reasoning, thinkingOnly: item.thinkingOnly, latest, turnStart, failure: failureOf(message), followUps: [], cards: [], quiet: false });
	});
	return foldCards(entries);
}

/** How long a reply took after the person asked: "9s", "2m 5s". */
export function turnDuration(askedAt: string | null, repliedAt: string): string | undefined {
	if (!askedAt) return undefined;
	const seconds = Math.max(0, Math.floor((Date.parse(repliedAt) - Date.parse(askedAt)) / 1000));
	return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}
