import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { SandboxRuntime } from './sandbox.svelte';

const SECRET = 'https://boat.example/p/s3cr3t-token';
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

type Status = { runtime: string; available: boolean; state: string; phase: string; preview_url: string | null; error: string | null };
let status: Status;
let logs: (offset: number) => { text: string; next_offset: number };
let requests: { path: string; method: string }[];

beforeEach(() => {
	vi.useFakeTimers();
	requests = [];
	status = { runtime: 'sandbox', available: true, state: 'running', phase: 'running', preview_url: SECRET, error: null };
	logs = (offset) => ({ text: '', next_offset: offset });
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init?: RequestInit) => {
			const path = String(url).replace('/projects/p1/sandbox/', '');
			requests.push({ path, method: init?.method ?? 'GET' });
			if (path === 'status') return json(status);
			if (path.startsWith('logs')) return json(logs(Number(new URLSearchParams(path.split('?')[1]).get('offset'))));
			return new Response(null, { status: 204 });
		}),
	);
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

const count = (path: string, method = 'GET') => requests.filter((one) => one.path.split('?')[0] === path && one.method === method).length;

describe('the sandbox runtime', () => {
	it('shows a running sandbox’s preview and keeps it awake while open', async () => {
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect([runtime.phase, runtime.previewUrl]).toEqual(['running', SECRET]);
		expect(count('beat')).toBe(1);
		await vi.advanceTimersByTimeAsync(20_000);
		expect(count('beat')).toBe(2);
		expect(count('start', 'POST')).toBe(0);
		runtime.stop();
		const after = requests.length;
		await vi.advanceTimersByTimeAsync(120_000);
		expect(requests.length).toBe(after);
		expect(count('stop', 'POST')).toBe(0);
	});

	it('wakes an idle sandbox once, and follows it until its preview is up', async () => {
		status = { ...status, state: 'none', phase: 'idle', preview_url: null };
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		await runtime.start();
		expect(count('start', 'POST')).toBe(1);
		expect(runtime.phase).toBe('waking');
		status = { ...status, state: 'running', phase: 'running', preview_url: null };
		await vi.advanceTimersByTimeAsync(5_000);
		await vi.waitFor(() => expect(runtime.phase).toBe('starting'));
		status = { ...status, preview_url: SECRET };
		await vi.advanceTimersByTimeAsync(5_000);
		await vi.waitFor(() => expect([runtime.phase, runtime.previewUrl]).toEqual(['running', SECRET]));
		expect(count('beat')).toBe(1);
		runtime.stop();
	});

	it('says why it failed, and tries again when asked', async () => {
		status = { ...status, state: 'error', phase: 'error', preview_url: null, error: 'npm install failed' };
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect([runtime.phase, runtime.error]).toEqual(['failed', 'npm install failed']);
		status = { ...status, state: 'running', phase: 'running', preview_url: SECRET, error: null };
		await runtime.retry();
		expect(count('start', 'POST')).toBe(1);
		expect([runtime.phase, runtime.error]).toEqual(['running', null]);
		runtime.stop();
	});

	it('fails with the backend’s reason when the sandbox can’t start', async () => {
		status = { ...status, state: 'none', phase: 'idle', preview_url: null };
		vi.mocked(fetch).mockImplementation(async (url, init) => {
			const path = String(url).replace('/projects/p1/sandbox/', '');
			if (init?.method === 'POST') return new Response('the sandbox isn’t available right now', { status: 409 });
			return path === 'status' ? json(status) : new Response(null, { status: 204 });
		});
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect([runtime.phase, runtime.error]).toEqual(['failed', 'the sandbox isn’t available right now']);
		runtime.stop();
	});

	it('appends the log from the offset it left off at', async () => {
		logs = (offset) => (offset === 0 ? { text: '$ npm run dev\n', next_offset: 14 } : offset === 14 ? { text: 'ready\n', next_offset: 20 } : { text: '', next_offset: offset });
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect(runtime.log).toBe('$ npm run dev\n');
		await vi.advanceTimersByTimeAsync(5_000);
		await vi.waitFor(() => expect(runtime.log).toBe('$ npm run dev\nready\n'));
		expect(requests.map((one) => one.path)).toContain('logs?offset=14');
		runtime.stop();
	});

	it('never puts the preview address anywhere but its own field', async () => {
		const spies = (['log', 'info', 'warn', 'error', 'debug'] as const).map((key) => vi.spyOn(console, key).mockImplementation(() => {}));
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		await vi.advanceTimersByTimeAsync(5_000);
		expect(runtime.previewUrl).toBe(SECRET);
		expect(runtime.log).not.toContain('s3cr3t');
		expect(runtime.error ?? '').not.toContain('s3cr3t');
		expect(requests.some((one) => one.path.includes('s3cr3t'))).toBe(false);
		for (const spy of spies) expect(JSON.stringify(spy.mock.calls)).not.toContain('s3cr3t');
		runtime.stop();
	});

	it('doesn’t read the same offset twice when a poll lands during a slow read', async () => {
		let release: () => void = () => {};
		const gate = new Promise<void>((resolve) => (release = resolve));
		let served = 0;
		vi.mocked(fetch).mockImplementation(async (url) => {
			const path = String(url).replace('/projects/p1/sandbox/', '');
			if (path === 'status') return json(status);
			if (path.startsWith('logs')) {
				served += 1;
				await gate;
				return json({ text: 'once\n', next_offset: 5 });
			}
			return new Response(null, { status: 204 });
		});
		const runtime = new SandboxRuntime('p1');
		const started = runtime.start();
		await vi.advanceTimersByTimeAsync(5_000);
		release();
		await started;
		await vi.advanceTimersByTimeAsync(0);
		expect(served).toBe(1);
		expect(runtime.log).toBe('once\n');
		runtime.stop();
	});
});
