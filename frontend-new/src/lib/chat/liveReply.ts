// The reply as the crew writes it, from the chat's live stream (`Delta` envelopes carrying the
// model's own stream events, serde's external tagging: `{"TextDelta": {"index": 0, "text": "…"}}`).
// Text and thinking are kept apart: the thinking streams in the reasoning rail, the text below it;
// tool calls show as the live step instead. A new model call in the same turn (after a tool)
// starts over, as its earlier text is posted on its own.
// Ported from the old app's liveReply (frontend/src/lib/liveReply.ts), with the thinking added.

export type LiveReply = { text: string; thinking: string; textBlocks: number[]; ended: boolean };

export const emptyStream = (): LiveReply => ({ text: '', thinking: '', textBlocks: [], ended: false });

type Tagged = Record<string, Record<string, unknown> | undefined>;

export function applyStreamEvent(reply: LiveReply, event: unknown): LiveReply {
	if (!event || typeof event !== 'object') return reply;
	const tagged = event as Tagged;
	const base = () => (reply.ended ? emptyStream() : reply);
	const start = tagged.ContentBlockStart;
	if (start) {
		if (start.block !== 'Text') return reply;
		const fresh = base();
		return { ...fresh, textBlocks: [...fresh.textBlocks, Number(start.index)] };
	}
	const delta = tagged.TextDelta;
	if (delta && typeof delta.text === 'string') {
		const fresh = base();
		const index = Number(delta.index);
		// Some providers send text without announcing the block first.
		const textBlocks = fresh.textBlocks.includes(index) ? fresh.textBlocks : [...fresh.textBlocks, index];
		return { ...fresh, textBlocks, text: fresh.text + delta.text };
	}
	const thinking = tagged.ThinkingDelta;
	if (thinking && typeof thinking.text === 'string') {
		const fresh = base();
		return { ...fresh, thinking: fresh.thinking + thinking.text };
	}
	if (tagged.Done) return { ...reply, ended: true };
	return reply;
}
