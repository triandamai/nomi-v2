// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@webcontainer/api', () => ({ WebContainer: { boot: async () => ({}) } }));

const { SandboxRuntime, toRuntimePhase } = await import('./sandboxRuntime.svelte');
const { createRuntime, forceBrowserRuntime, browserRuntimeForced } = await import('./runtimeChooser');
const { ProjectRuntime } = await import('./projectRuntime.svelte');

const SECRET = 'https://boat.example/p/s3cr3t-token';
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

type Status = { runtime: string; available: boolean; phase: string; preview_url: string | null; error: string | null };
let status: Status;
let requests: { path: string; method: string }[];

beforeEach(() => {
	vi.useFakeTimers();
	requests = [];
	status = { runtime: 'sandbox', available: true, phase: 'running', preview_url: SECRET, error: null };
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init?: RequestInit) => {
			const path = String(url).replace('/projects/p1/api/', '');
			requests.push({ path, method: init?.method ?? 'GET' });
			if (path === 'sandbox') return json(status);
			if (path.startsWith('sandbox/logs')) return json({ text: '', next_offset: 0 });
			return json({});
		}),
	);
});

afterEach(() => {
	vi.useRealTimers();
	vi.unstubAllGlobals();
});

const beats = () => requests.filter((r) => r.path.split('?')[0] === 'runtime').length;

describe('sandbox runtime', () => {
	it('heartbeat_tells_the_backend_it_is_the_sandbox_page', async () => {
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		const heartbeats = requests.filter((r) => r.path.split('?')[0] === 'runtime');
		expect(heartbeats.length).toBeGreaterThan(0);
		// Only the sandbox page says so; the browser runtime's check-in never does, so a fallback page claims Koda's commands.
		expect(heartbeats.every((r) => new URLSearchParams(r.path.split('?')[1]).get('runtime') === 'sandbox')).toBe(true);
		await runtime.stop();
	});

	it('a_running_sandbox_whose_preview_failed_shows_the_failure', async () => {
		status = { ...status, phase: 'running', preview_url: null, error: 'installing dependencies failed: ERESOLVE' };
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect(runtime.phase).toBe('failed');
		expect(runtime.error).toContain('ERESOLVE');
		await runtime.stop();
	});

	it('picks_sandbox_runtime_when_available', async () => {
		expect(await createRuntime('p1')).toBeInstanceOf(SandboxRuntime);
	});

	it('falls_back_to_browser_runtime_when_unavailable', async () => {
		status.available = false;
		const unavailable = await createRuntime('p1');
		expect(unavailable).toBeInstanceOf(ProjectRuntime);
		expect(unavailable.log).toContain('browser');
		status = { ...status, runtime: 'webcontainer', available: true };
		expect(await createRuntime('p1')).toBeInstanceOf(ProjectRuntime);
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 500)));
		expect(await createRuntime('p1')).toBeInstanceOf(ProjectRuntime);
	});

	it('never_frames_a_preview_address_that_is_not_https', async () => {
		for (const bad of ['javascript:alert(1)', 'http://x.on.boat.dev/?_token=t', 'data:text/html,hi', 'not a url']) {
			status = { ...status, phase: 'running', preview_url: bad };
			const runtime = new SandboxRuntime('p1');
			await runtime.start();
			expect(runtime.previewUrl, bad).toBeNull();
			expect(runtime.phase, bad).toBe('starting');
			await runtime.stop();
		}
	});

	it('maps_backend_phases_to_runtime_phases', () => {
		expect(toRuntimePhase('idle', null)).toBe('waiting');
		expect(toRuntimePhase('stopped', null)).toBe('waiting');
		expect(toRuntimePhase('waking', null)).toBe('waking');
		expect(toRuntimePhase('running', SECRET)).toBe('running');
		expect(toRuntimePhase('running', null)).toBe('starting');
		expect(toRuntimePhase('stopping', null)).toBe('stopped');
		expect(toRuntimePhase('error', null)).toBe('failed');
	});

	it('follows the backend, starts an idle sandbox and reads its events', async () => {
		status = { ...status, phase: 'idle', preview_url: null };
		const runtime = new SandboxRuntime('p1');
		let emit: (raw: string) => void = () => {};
		runtime.listen((l) => ((emit = l), () => {}));
		await runtime.start();
		expect(requests.some((r) => r.path === 'sandbox/start' && r.method === 'POST')).toBe(true);
		expect(runtime.phase).toBe('waiting');
		status = { ...status, phase: 'running', preview_url: SECRET };
		emit('{"kind":"SandboxChanged","phase":"running"}');
		await vi.advanceTimersByTimeAsync(0);
		expect(runtime.phase).toBe('running');
		expect(runtime.previewUrl).toBe(SECRET);
		status = { ...status, phase: 'error', error: 'boom' };
		await vi.advanceTimersByTimeAsync(5000);
		expect(runtime.phase).toBe('failed');
		expect(runtime.error).toBe('boom');
		await runtime.stop();
	});

	it('heartbeat_continues_while_open_and_stops_on_stop', async () => {
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		const first = beats();
		expect(first).toBeGreaterThanOrEqual(1);
		await vi.advanceTimersByTimeAsync(60_000);
		expect(beats()).toBeGreaterThan(first);
		await runtime.stop();
		const after = requests.length;
		await vi.advanceTimersByTimeAsync(120_000);
		expect(requests.length).toBe(after);
		expect(runtime.phase).toBe('stopped');
	});

	it('writeFile_and_removeFile_do_not_call_the_api', async () => {
		const runtime = new SandboxRuntime('p1');
		await runtime.writeFile('a.txt', 'x');
		await runtime.removeFile('a.txt');
		expect(requests).toEqual([]);
	});

	it('preview_url_is_not_logged_or_kept_in_the_dom_text', async () => {
		const spies = (['log', 'info', 'warn', 'error', 'debug'] as const).map((k) => vi.spyOn(console, k).mockImplementation(() => {}));
		status.phase = 'error';
		status.error = null;
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		status = { ...status, phase: 'running', error: null };
		await vi.advanceTimersByTimeAsync(5000);
		expect(runtime.previewUrl).toBe(SECRET);
		expect(runtime.log).not.toContain('s3cr3t');
		expect(runtime.error ?? '').not.toContain('s3cr3t');
		expect(document.body.textContent).not.toContain('s3cr3t');
		for (const spy of spies) expect(JSON.stringify(spy.mock.calls)).not.toContain('s3cr3t');
		// Never sent in another request's address either.
		expect(requests.some((r) => r.path.includes('s3cr3t'))).toBe(false);
		await runtime.stop();
	});

	it('appends logs from the offset it left off at', async () => {
		let served = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				const path = String(url).replace('/projects/p1/api/', '');
				if (path === 'sandbox') return json(status);
				if (path.startsWith('sandbox/logs')) {
					requests.push({ path, method: 'GET' });
					return json({ text: served++ ? 'two\n' : 'one\n', next_offset: served * 4 });
				}
				return json({});
			}),
		);
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		await vi.advanceTimersByTimeAsync(5000);
		expect(runtime.log).toContain('one');
		expect(runtime.log).toContain('two');
		expect(requests.map((r) => r.path)).toContain('sandbox/logs?offset=4');
		await runtime.stop();
	});

	it('a second start() does not double the heartbeat or the first sandbox start', async () => {
		status = { ...status, phase: 'idle', preview_url: null };
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		await runtime.start();
		expect(requests.filter((r) => r.path === 'sandbox/start')).toHaveLength(1);
		await vi.advanceTimersByTimeAsync(20_000);
		expect(beats()).toBeLessThanOrEqual(3);
		await runtime.stop();
	});

	it('overlapping refreshes do not duplicate the log text', async () => {
		let release: () => void = () => {};
		const gate = new Promise<void>((r) => (release = r));
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => {
				const path = String(url).replace('/projects/p1/api/', '');
				if (path === 'sandbox') return json(status);
				if (path.startsWith('sandbox/logs')) {
					await gate;
					return json({ text: 'once\n', next_offset: 5 });
				}
				return json({});
			}),
		);
		const runtime = new SandboxRuntime('p1');
		let emit: (raw: string) => void = () => {};
		runtime.listen((l) => ((emit = l), () => {}));
		const starting = runtime.start();
		emit('{"kind":"SandboxChanged"}');
		emit('{"kind":"SandboxChanged"}');
		await vi.advanceTimersByTimeAsync(0);
		release();
		await starting;
		await vi.advanceTimersByTimeAsync(0);
		expect(runtime.log.split('once').length - 1).toBe(1);
		await runtime.stop();
	});

	it('retry asks the backend to start the sandbox again and clears the error', async () => {
		status = { ...status, phase: 'error', preview_url: null, error: 'boom' };
		const runtime = new SandboxRuntime('p1');
		await runtime.start();
		expect(runtime.phase).toBe('failed');
		status = { ...status, phase: 'waking', error: null };
		await runtime.retry();
		expect(requests.some((r) => r.path === 'sandbox/start' && r.method === 'POST')).toBe(true);
		expect(runtime.error).toBeNull();
		expect(runtime.phase).toBe('waking');
		await runtime.stop();
	});

	it('forcing the browser runtime skips the sandbox for this page session', async () => {
		expect(browserRuntimeForced('p1')).toBe(false);
		forceBrowserRuntime('p1');
		expect(browserRuntimeForced('p1')).toBe(true);
		expect(browserRuntimeForced('p2')).toBe(false);
		requests.length = 0;
		expect(await createRuntime('p1')).toBeInstanceOf(ProjectRuntime);
		expect(requests).toEqual([]);
	});
});
