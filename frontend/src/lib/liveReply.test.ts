import { describe, expect, it } from 'vitest';
import { applyStreamEvent, emptyStream } from './liveReply';

const run = (events: unknown[]) => events.reduce(applyStreamEvent, emptyStream());

describe('the live reply', () => {
	it('collects the text the model writes, leaving thinking and tool calls out', () => {
		const reply = run([
			{ ContentBlockStart: { index: 0, block: 'Thinking' } },
			{ ThinkingDelta: { index: 0, text: 'hmm' } },
			{ ContentBlockStart: { index: 1, block: 'Text' } },
			{ TextDelta: { index: 1, text: 'Rent is ' } },
			{ TextDelta: { index: 1, text: 'Rp 2 jt.' } },
			{ ToolInputDelta: { index: 2, partial_json: '{' } },
		]);
		expect(reply.text).toBe('Rent is Rp 2 jt.');
	});

	it('starts over for the next model call in the same turn', () => {
		const reply = run([
			{ TextDelta: { index: 0, text: 'Let me check.' } },
			{ Done: { stop_reason: 'ToolUse', input_tokens: 1, output_tokens: 1 } },
			{ ContentBlockStart: { index: 0, block: 'Text' } },
			{ TextDelta: { index: 0, text: 'Found it.' } },
		]);
		expect(reply.text).toBe('Found it.');
	});
});
