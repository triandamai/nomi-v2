// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// A WebContainer stand-in: a file system in a Map and processes the test finishes by hand.
const wc = vi.hoisted(() => {
	type Proc = { command: string; args: string[]; env: Record<string, string>; exit: Promise<number>; finish: (code: number) => void; kill: ReturnType<typeof import('vitest').vi.fn>; say: (text: string) => void };
	const state = {
		files: new Map<string, string>(),
		dirs: new Set<string>(),
		procs: [] as Proc[],
		listeners: {} as Record<string, (...args: unknown[]) => void>,
		torn: 0,
		bootFails: false,
		exitCodes: {} as Record<string, number>,
	};
	const container = {
		on(event: string, listener: (...args: unknown[]) => void) {
			state.listeners[event] = listener;
		},
		async mount(tree: Record<string, unknown>) {
			const walk = (node: Record<string, unknown>, prefix: string) => {
				for (const [name, entry] of Object.entries(node)) {
					const e = entry as { file?: { contents: string }; directory?: Record<string, unknown> };
					if (e.file) state.files.set(prefix + name, e.file.contents);
					else if (e.directory) walk(e.directory, `${prefix}${name}/`);
				}
			};
			walk(tree, '');
		},
		fs: {
			async mkdir(dir: string) {
				state.dirs.add(dir);
			},
			async writeFile(path: string, content: string) {
				state.files.set(path, content);
			},
			async readFile(path: string) {
				if (!state.files.has(path)) throw new Error('ENOENT');
				return state.files.get(path)!;
			},
			async readdir(dir: string) {
				if (!state.dirs.has(dir)) throw new Error('ENOENT');
				return ['pkg'];
			},
			async rm(path: string) {
				state.files.delete(path);
			},
		},
		async spawn(command: string, args: string[], options: { env: Record<string, string> }) {
			let finish!: (code: number) => void;
			const exit = new Promise<number>((resolve) => (finish = resolve));
			let controller!: ReadableStreamDefaultController<string>;
			const output = new ReadableStream<string>({ start: (c) => (controller = c) });
			const proc = { command, args, env: options.env, exit, finish, kill: vi.fn(), say: (text: string) => controller.enqueue(text), output };
			state.procs.push(proc);
			const key = [command, ...args].join(' ');
			if (key in state.exitCodes) {
				if (key === 'npm install' && state.exitCodes[key] === 0) state.dirs.add('node_modules');
				queueMicrotask(() => {
					proc.say(`ran ${key}\n`);
					finish(state.exitCodes[key]);
				});
			}
			return proc;
		},
		teardown() {
			state.torn += 1;
		},
	};
	return { state, container };
});

vi.mock('@webcontainer/api', () => ({
	WebContainer: {
		boot: async () => {
			if (wc.state.bootFails) throw new Error('boot failed');
			return wc.container;
		},
	},
}));

const { ProjectRuntime } = await import('./projectRuntime.svelte');

type Handler = (path: string, init?: RequestInit) => Response | Promise<Response> | undefined;
let backend: Handler;
const requests: { path: string; method: string; body?: unknown }[] = [];
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

/** The check-in long-poll: answers once with `first`, then waits until the runtime stops. */
function checkIns(first: unknown) {
	let answered = false;
	return (init?: RequestInit) => {
		if (!answered) {
			answered = true;
			return json(first);
		}
		return new Promise<Response>((_, reject) => init?.signal?.addEventListener('abort', () => reject(new Error('aborted'))));
	};
}

beforeEach(() => {
	wc.state.files.clear();
	wc.state.dirs.clear();
	wc.state.procs = [];
	wc.state.listeners = {};
	wc.state.bootFails = false;
	wc.state.exitCodes = { 'npm install': 0 };
	requests.length = 0;
	Object.defineProperty(window, 'crossOriginIsolated', { value: true, configurable: true });
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init?: RequestInit) => {
			const path = String(url).replace('/projects/p1/api/', '');
			requests.push({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(String(init.body)) : undefined });
			const answer = await backend(path, init);
			return answer ?? json({}, 404);
		}),
	);
});

afterEach(() => {
	vi.unstubAllGlobals();
});

const snapshot = (files: { path: string; content: string }[], version = 1) =>
	json({ files_version: version, files: files.map((f) => ({ ...f, updated_at: `v${version}` })) });

async function until(check: () => boolean) {
	await vi.waitFor(() => expect(check()).toBe(true), { timeout: 2000 });
}

describe('ProjectRuntime', () => {
	it('falls back when the page is not cross-origin isolated', async () => {
		Object.defineProperty(window, 'crossOriginIsolated', { value: false, configurable: true });
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		expect(runtime.phase).toBe('unsupported');
	});

	it('fails with the reason when booting fails', async () => {
		wc.state.bootFails = true;
		const runtime = new ProjectRuntime('fresh');
		await runtime.start();
		expect(runtime.phase).toBe('failed');
		expect(runtime.error).toBe('boot failed');
		await runtime.stop();
		wc.state.bootFails = false;
	});

	it('loads, installs, reports the check, starts the dev server and shows its preview', async () => {
		const poll = checkIns({ files_version: 1, runs: [] });
		backend = (path, init) => {
			if (path === 'snapshot') return snapshot([{ path: 'package.json', content: '{"scripts":{"dev":"vite"}}' }, { path: 'src/main.ts', content: 'x' }]);
			if (path === 'relay/preview') return json({}, 404);
			if (path === 'runtime/check') return json({});
			if (path.startsWith('runtime?')) return poll(init);
		};
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		await until(() => runtime.phase === 'starting');
		expect(runtime.files.map((f) => f.path)).toEqual(['package.json', 'src/main.ts']);
		expect(requests.find((r) => r.path === 'runtime/check')?.body).toMatchObject({ files_version: 1, ok: true });
		expect(wc.state.procs.map((p) => [p.command, ...p.args].join(' '))).toEqual(['npm install', 'npm run dev']);
		expect(wc.state.procs[1].env).toMatchObject({ NOMI_PREVIEW: '1' });
		wc.state.listeners['server-ready'](5173, 'https://preview.local');
		expect(runtime.phase).toBe('running');
		expect(runtime.previewUrl).toBe('https://preview.local');
		expect(runtime.previewPort).toBe(5173);
		expect(runtime.log).toContain('$ npm install');

		// The editor's own changes go straight in.
		await runtime.writeFile('src/new/file.ts', 'y');
		expect(wc.state.files.get('src/new/file.ts')).toBe('y');
		expect(wc.state.dirs.has('src/new')).toBe(true);
		await runtime.removeFile('src/main.ts');
		expect(runtime.files.map((f) => f.path)).toEqual(['package.json']);

		// The dev server dying is a failure the person can restart from.
		wc.state.procs[1].finish(1);
		await until(() => runtime.phase === 'failed');
		expect(runtime.error).toContain('exit code 1');
		await runtime.restart();
		expect(runtime.phase).toBe('starting');
		expect(runtime.error).toBeNull();
		expect(wc.state.procs.filter((p) => p.args.join(' ') === 'run dev')).toHaveLength(2);

		await runtime.stop();
		expect(runtime.phase).toBe('stopped');
		wc.state.listeners['server-ready'](1, 'https://late');
		expect(runtime.previewUrl).toBeNull();
	});

	it('waits for package.json, then syncs it in and runs what Koda asks', async () => {
		const poll = checkIns({ files_version: 2, runs: [{ id: 'r1', command: 'npm', args: ['run', 'build'] }] });
		wc.state.exitCodes['npm run build'] = 0;
		backend = (path, init) => {
			if (path === 'snapshot') return snapshot([{ path: 'README.md', content: 'hi' }]);
			if (path.startsWith('runtime?')) return poll(init);
			if (path === 'manifest') return json({ files_version: 2, files: [{ path: 'package.json', size_bytes: 2, updated_at: 'v2' }] });
			if (path === 'files/package.json') return new Response('{}');
			if (path === 'relay/preview') return json({ url: 'https://relay.dev', anon_key: 'anon', service_key: 'svc' });
			return json({});
		};
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		expect(['waiting', 'installing', 'starting']).toContain(runtime.phase);
		await until(() => requests.some((r) => r.path === 'runtime/runs/r1'));
		expect(requests.find((r) => r.path === 'runtime/runs/r1')?.body).toMatchObject({ exit_code: 0 });
		await until(() => wc.state.procs.some((p) => p.args.join(' ') === 'run dev'));
		expect(wc.state.files.has('README.md')).toBe(false);
		expect(wc.state.files.get('package.json')).toBe('{}');
		const dev = wc.state.procs.find((p) => p.args.join(' ') === 'run dev')!;
		expect(dev.env).toMatchObject({ PUBLIC_NOMI_URL: 'https://relay.dev', NOMI_SERVICE_KEY: 'svc' });
		expect(runtime.running).toBeNull();
		await runtime.stop();
	});

	it('marks a failed install for Koda once', async () => {
		wc.state.exitCodes['npm install'] = 1;
		const poll = checkIns({ files_version: 1, runs: [] });
		backend = (path, init) => {
			if (path === 'snapshot') return snapshot([{ path: 'package.json', content: '{}' }]);
			if (path.startsWith('runtime?')) return poll(init);
			if (path === 'relay/preview') return json({}, 500);
			return json({});
		};
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		await until(() => runtime.phase === 'failed');
		expect(runtime.checkFailed).toBe(true);
		expect(runtime.error).toContain('npm install failed');
		expect(requests.find((r) => r.path === 'runtime/check')?.body).toMatchObject({ ok: false });
		await runtime.stop();
	});

	it('fails to open a project whose files cannot be read', async () => {
		backend = () => json({}, 503);
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		expect(runtime.phase).toBe('failed');
		expect(runtime.error).toContain('503');
		await runtime.stop();
	});

	it('picks up a dev backend that finished starting and restarts the preview with it', async () => {
		const poll = checkIns({ files_version: 1, runs: [] });
		let relay: Response = json({}, 404);
		backend = (path, init) => {
			if (path === 'snapshot') return snapshot([{ path: 'package.json', content: '{}' }]);
			if (path.startsWith('runtime?')) return poll(init);
			if (path === 'relay/preview') return relay;
			return json({});
		};
		const runtime = new ProjectRuntime('p1');
		await runtime.start();
		await until(() => wc.state.procs.some((p) => p.args.join(' ') === 'run dev'));
		relay = json({ url: 'https://relay.up', anon_key: 'k' });
		await runtime.refreshRelay();
		const devs = wc.state.procs.filter((p) => p.args.join(' ') === 'run dev');
		expect(devs).toHaveLength(2);
		expect(devs[0].kill).toHaveBeenCalled();
		expect(devs[1].env.PUBLIC_NOMI_URL).toBe('https://relay.up');
		await runtime.stop();
	});

	it('ignores editor writes before it has booted', async () => {
		const runtime = new ProjectRuntime('p1');
		await runtime.writeFile('a', 'b');
		await runtime.removeFile('a');
		expect(wc.state.files.size).toBe(0);
	});
});
