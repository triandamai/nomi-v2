// Build → Workspace: the chat/preview split. The chat stays between 360 and 720px, the preview
// never goes below 480, and a double-click resets the chat to 520.

export const SPLIT = { min: 360, max: 720, reset: 520, previewMin: 480 } as const;

/** The chat's width for a wanted width, given the room the split has. */
export function chatWidth(wanted: number, available: number): number {
	const max = Math.max(SPLIT.min, Math.min(SPLIT.max, available - SPLIT.previewMin));
	return Math.round(Math.min(max, Math.max(SPLIT.min, wanted)));
}

/** The chat's width after a key on the handle: arrows by 16px (64 with Shift), Home and End to the ends. */
export function nudge(width: number, key: string, shift: boolean, available: number): number {
	const step = shift ? 64 : 16;
	if (key === 'ArrowLeft') return chatWidth(width - step, available);
	if (key === 'ArrowRight') return chatWidth(width + step, available);
	if (key === 'Home') return chatWidth(SPLIT.min, available);
	if (key === 'End') return chatWidth(SPLIT.max, available);
	return width;
}
