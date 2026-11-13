import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { settledRefresh } from './settle';

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe('settledRefresh', () => {
	it('runs once after a burst has been quiet for the wait', () => {
		const run = vi.fn();
		const refresh = settledRefresh(run, () => false, { wait: 400 });
		refresh.schedule();
		vi.advanceTimersByTime(300);
		refresh.schedule();
		vi.advanceTimersByTime(399);
		expect(run).not.toHaveBeenCalled();
		vi.advanceTimersByTime(1);
		expect(run).toHaveBeenCalledOnce();
	});

	it('waits out a navigation in flight, so the reload doesn’t cancel it', () => {
		const run = vi.fn();
		let navigating = true;
		const refresh = settledRefresh(run, () => navigating, { wait: 400, retry: 100 });
		refresh.schedule();
		vi.advanceTimersByTime(1_000);
		expect(run).not.toHaveBeenCalled();
		navigating = false;
		vi.advanceTimersByTime(100);
		expect(run).toHaveBeenCalledOnce();
	});

	it('does nothing once cancelled', () => {
		const run = vi.fn();
		const refresh = settledRefresh(run, () => false);
		refresh.schedule();
		refresh.cancel();
		vi.advanceTimersByTime(5_000);
		expect(run).not.toHaveBeenCalled();
	});
});
