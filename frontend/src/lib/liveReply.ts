// The reply as the crew writes it, from the chat's live stream (`Delta` envelopes carrying the
// model's own stream events, serde's external tagging: `{"TextDelta": {"index": 0, "text": "…"}}`).
// Only text blocks count: thinking and tool calls show as the live step instead. A new model call
// in the same turn (after a tool) starts the text over, as its earlier text is posted on its own.

export type LiveReply = { text: string; textBlocks: number[]; ended: boolean };

export const emptyStream = (): LiveReply => ({ text: '', textBlocks: [], ended: false });

type Tagged = Record<string, Record<string, unknown> | undefined>;

export function applyStreamEvent(reply: LiveReply, event: unknown): LiveReply {
	if (!event || typeof event !== 'object') return reply;
	const tagged = event as Tagged;
	const start = tagged.ContentBlockStart;
	if (start) {
		const index = Number(start.index);
		if (start.block !== 'Text') return reply;
		const fresh = reply.ended ? emptyStream() : reply;
		return { ...fresh, textBlocks: [...fresh.textBlocks, index] };
	}
	const delta = tagged.TextDelta;
	if (delta && typeof delta.text === 'string') {
		const base = reply.ended ? emptyStream() : reply;
		const index = Number(delta.index);
		// Some providers send text without announcing the block first.
		const textBlocks = base.textBlocks.includes(index) ? base.textBlocks : [...base.textBlocks, index];
		return { ...base, textBlocks, text: base.text + delta.text };
	}
	if (tagged.Done) return { ...reply, ended: true };
	return reply;
}
