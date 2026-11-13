// Folds an agent's reasoning messages into the reply they led to, so the thread shows one
// reasoning rail on that reply instead of a separate message per thinking step.
// Ported from the old app's reasoning (frontend/src/lib/reasoning.ts).
import type { ContentBlock } from '$lib/adapters/api';

const LEGACY_PREFIX = '🧠 ';

export type Groupable = {
	id: string;
	sender: 'user' | 'assistant';
	content: string;
	content_blocks: ContentBlock[] | null;
	created_at: string;
	agent_display_name: string | null;
};

/** The thinking text if `message` is a reasoning step, else null. Older steps were plain text
 * starting with 🧠 and carry no block. */
export function reasoningText(message: Groupable): string | null {
	if (message.sender !== 'assistant') return null;
	const block = message.content_blocks?.[0];
	if (block?.kind === 'reasoning' && typeof block.text === 'string') return block.text;
	if (!message.content_blocks?.length && message.content.startsWith(LEGACY_PREFIX)) return message.content.slice(LEGACY_PREFIX.length);
	return null;
}

export interface ThreadItem<T extends Groupable> {
	message: T;
	/** Thinking steps that led to this message, oldest first. */
	reasoning: string[];
	/** When the first of them was posted. */
	thinkingSince: string | null;
	/** True when the item is only thinking so far: no reply from that agent has landed yet. */
	thinkingOnly: boolean;
}

export function groupReasoning<T extends Groupable>(messages: T[]): ThreadItem<T>[] {
	const items: ThreadItem<T>[] = [];
	// Steps waiting for their reply, per agent (display name; null = Nomi).
	const pending = new Map<string | null, { steps: string[]; first: T }>();

	for (const message of messages) {
		const text = reasoningText(message);
		const agent = message.agent_display_name ?? null;
		if (text !== null) {
			const waiting = pending.get(agent);
			if (waiting) waiting.steps.push(text);
			else pending.set(agent, { steps: [text], first: message });
			continue;
		}
		const waiting = message.sender === 'assistant' ? pending.get(agent) : undefined;
		if (waiting) pending.delete(agent);
		items.push({ message, reasoning: waiting?.steps ?? [], thinkingSince: waiting?.first.created_at ?? null, thinkingOnly: false });
	}

	// Thinking with no reply yet (still working, or stopped) stays visible, collapsed, where it began.
	for (const { steps, first } of pending.values()) {
		const at = items.findIndex((item) => item.message.created_at > first.created_at);
		const item = { message: first, reasoning: steps, thinkingSince: first.created_at, thinkingOnly: true };
		if (at === -1) items.push(item);
		else items.splice(at, 0, item);
	}
	return items;
}
