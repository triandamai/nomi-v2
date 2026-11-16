// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@webcontainer/api', () => ({ WebContainer: { boot: async () => ({}) } }));

const { previewReady, startWhenReady, READY_POLL_MS } = await import('./runtimeChooser');

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
let ready: boolean | undefined;
let fails = false;

beforeEach(() => {
	vi.useFakeTimers();
	ready = false;
	fails = false;
	vi.stubGlobal(
		'fetch',
		vi.fn(async () => (fails ? json({}, 500) : json({ runtime: 'browser', available: false, phase: 'idle', preview_url: null, error: null, preview_ready: ready }))),
	);
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

describe('waiting for the first build before previewing', () => {
	it('reads whether there is a build to preview, and previews as before when it cannot tell', async () => {
		expect(await previewReady('p1')).toBe(false);
		ready = true;
		expect(await previewReady('p1')).toBe(true);
		ready = undefined;
		expect(await previewReady('p1')).toBe(true);
		fails = true;
		expect(await previewReady('p1')).toBe(true);
	});

	it('starts nothing until the first build is done, then starts once', async () => {
		const start = vi.fn();
		const waiting: boolean[] = [];
		startWhenReady('p1', start, (w) => waiting.push(w));
		await vi.advanceTimersByTimeAsync(0);
		expect(start).not.toHaveBeenCalled();
		expect(waiting).toEqual([true]);

		await vi.advanceTimersByTimeAsync(READY_POLL_MS);
		expect(start).not.toHaveBeenCalled();

		ready = true;
		await vi.advanceTimersByTimeAsync(READY_POLL_MS);
		expect(start).toHaveBeenCalledTimes(1);
		expect(waiting).toEqual([true, false]);
		await vi.advanceTimersByTimeAsync(READY_POLL_MS * 3);
		expect(start).toHaveBeenCalledTimes(1);
	});

	it('starts straight away when there is already a build', async () => {
		ready = true;
		const start = vi.fn();
		const waiting: boolean[] = [];
		startWhenReady('p1', start, (w) => waiting.push(w));
		await vi.advanceTimersByTimeAsync(0);
		expect(start).toHaveBeenCalledTimes(1);
		expect(waiting).toEqual([]);
	});

	it('stops asking once cancelled', async () => {
		const start = vi.fn();
		const cancel = startWhenReady('p1', start, () => undefined);
		await vi.advanceTimersByTimeAsync(0);
		cancel();
		ready = true;
		await vi.advanceTimersByTimeAsync(READY_POLL_MS * 2);
		expect(start).not.toHaveBeenCalled();
	});
});
