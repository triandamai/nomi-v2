import { describe, expect, it } from 'vitest';
import { SPLIT, chatWidth, nudge } from './split';

// Build → Workspace: chat and preview share one screen. The chat stays between 360 and 720px,
// the preview never goes below 480, and the split resets to 520.

describe('the workspace split', () => {
	it('keeps the chat between its limits and leaves the preview 480px', () => {
		expect(SPLIT).toEqual({ min: 360, max: 720, reset: 520, previewMin: 480 });
		expect(chatWidth(300, 1600)).toBe(360);
		expect(chatWidth(900, 1600)).toBe(720);
		expect(chatWidth(700, 1100)).toBe(620);
		expect(chatWidth(500, 1600)).toBe(500);
	});

	it('moves by 16px with the arrow keys, 64 with Shift, and jumps with Home and End', () => {
		expect(nudge(520, 'ArrowRight', false, 1600)).toBe(536);
		expect(nudge(520, 'ArrowLeft', true, 1600)).toBe(456);
		expect(nudge(520, 'Home', false, 1600)).toBe(360);
		expect(nudge(520, 'End', false, 1600)).toBe(720);
		expect(nudge(520, 'a', false, 1600)).toBe(520);
	});
});
