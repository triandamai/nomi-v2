import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createLater } from './later';

describe('things done a little later, with a way back', () => {
	beforeEach(() => vi.useFakeTimers());
	afterEach(() => vi.useRealTimers());

	it('does it once the wait is over', () => {
		const later = createLater();
		const run = vi.fn();
		later.schedule('chat-1', run, 10_000);
		vi.advanceTimersByTime(9_999);
		expect(run).not.toHaveBeenCalled();
		vi.advanceTimersByTime(1);
		expect(run).toHaveBeenCalledOnce();
		expect(later.pending('chat-1')).toBe(false);
	});

	it('never does it once undone', () => {
		const later = createLater();
		const run = vi.fn();
		later.schedule('chat-1', run, 10_000);
		expect(later.pending('chat-1')).toBe(true);
		expect(later.undo('chat-1')).toBe(true);
		vi.advanceTimersByTime(20_000);
		expect(run).not.toHaveBeenCalled();
		expect(later.undo('chat-1')).toBe(false);
	});

	it('does everything still waiting at once when the page goes away', () => {
		const later = createLater();
		const first = vi.fn();
		const second = vi.fn();
		later.schedule('a', first, 10_000);
		later.schedule('b', second, 10_000);
		later.flush();
		expect(first).toHaveBeenCalledOnce();
		expect(second).toHaveBeenCalledOnce();
		vi.advanceTimersByTime(20_000);
		expect(first).toHaveBeenCalledOnce();
	});
});
