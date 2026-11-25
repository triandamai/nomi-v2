import type { WebContainer, WebContainerProcess } from '@webcontainer/api';
import { m } from '$lib/paraglide/messages';
import { diffManifest, needsForcedInstall, parentDir, previewEnv, toFileTree, watchExit, type ManifestEntry, type RelayPreviewAccess } from './device';
import type { SandboxPhase } from './sandbox';
import { appendOutput, capLog } from './terminal';

// Runs a project on the person's own device (WebContainer, in this tab) when the backend's sandbox
// is off or can't be used (the preview's "Use your device instead"): loads its files, installs it,
// starts its dev server for the preview, copies in Koda's changes as they're saved and runs the
// commands Koda asks for, sending the output back. Ported from the old app's ProjectRuntime.
// The page must be cross-origin isolated: the workspace asks for that with `?run=device` (hooks).
// Phases are the sandbox's, so the preview and terminal show it the same way.

type ClaimedRun = { id: string; command: string; args: string[]; timeout_seconds?: number };

// NODE_NO_WARNINGS: WebContainer's Node prints a WASI warning on every start, burying the app's output.
const PREVIEW_ENV = { NOMI_PREVIEW: '1', NODE_NO_WARNINGS: '1' };
/** How long a check-in waits for news (seconds, the backend's long poll). */
const CHECK_IN_WAIT_SECS = 20;
const COMMAND_TIMEOUT_MS = 120_000;
const RETRY_MS = 3000;
/** WebContainer loads from StackBlitz's servers: give up waiting after this long. */
const BOOT_TIMEOUT_MS = 60_000;
/** An install this quiet with node_modules in place is done, even if it hasn't exited. */
const INSTALL_QUIET_MS = 45_000;
/** The share of a command's output sent back to Koda. */
const OUTPUT_TAIL = 60_000;
const MS_PER_SECOND = 1000;
/** Koda's own timeout, less a margin so the result gets back before it gives up. */
const RUN_MARGIN_MS = 3000;
const RUN_MIN_MS = 5000;

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

// One WebContainer per page: booting a second one needs the first torn down.
let booted: Promise<WebContainer> | null = null;

export class DeviceRuntime {
	phase = $state<SandboxPhase>('loading');
	previewUrl = $state<string | null>(null);
	log = $state('');
	error = $state<string | null>(null);
	/** The device can't run it (the page isn't isolated, or the browser has no WebContainer). */
	unsupported = $state(false);

	#base: string;
	#wc: WebContainer | null = null;
	#dev: WebContainerProcess | null = null;
	#known = new Map<string, string>();
	#version = -1;
	#seen = -1;
	#queue: Promise<unknown> = Promise.resolve();
	#stopped = false;
	#abort = new AbortController();
	#checked = false;
	#env: Record<string, string> = PREVIEW_ENV;
	#relayLoaded = false;

	constructor(projectId: string) {
		this.#base = `/projects/${encodeURIComponent(projectId)}/device`;
	}

	async start() {
		if (typeof window === 'undefined' || !window.crossOriginIsolated) {
			this.unsupported = true;
			this.#fail(m.device_unsupported());
			return;
		}
		try {
			this.phase = 'waking';
			this.#append(`${m.device_booting()}\n`);
			const { WebContainer } = await import('@webcontainer/api');
			booted ??= WebContainer.boot({ coep: 'credentialless', workdirName: 'project' });
			const timedOut = sleep(BOOT_TIMEOUT_MS).then(() => {
				throw new Error(m.device_boot_timeout());
			});
			this.#wc = await Promise.race([booted, timedOut]);
			this.#wc.on('server-ready', (_port, url) => {
				if (this.#stopped) return;
				this.previewUrl = url;
				this.phase = 'running';
			});
			this.#wc.on('error', (problem) => this.#fail(problem.message));
			await this.#loadAll();
			void this.#loop();
			await this.#serial(() => this.#prepareIfReady());
		} catch (problem) {
			if (!this.#stopped) this.#fail(problem instanceof Error ? problem.message : String(problem));
		}
	}

	stop() {
		this.#stopped = true;
		this.#abort.abort();
		this.#dev?.kill();
		const current = booted;
		booted = null;
		void current?.then((wc) => wc.teardown()).catch(() => undefined);
	}

	/** The sandbox's live-link nudge: nothing to read here, the device is its own runtime. */
	async onSandboxChange() {}

	/** Starts the dev server again; one that never booted needs a fresh page. */
	async retry() {
		if (this.#stopped) return;
		if (!this.#wc) return location.reload();
		this.error = null;
		await this.#serial(() => this.#prepareIfReady());
	}

	#serial<T>(task: () => Promise<T>): Promise<T> {
		const next = this.#queue.then(task, task);
		this.#queue = next.catch(() => undefined);
		return next;
	}

	#fail(message: string) {
		this.error = message;
		this.phase = 'failed';
	}

	#append(text: string) {
		this.log = capLog(appendOutput(this.log, text));
	}

	#api(path: string, init?: RequestInit): Promise<Response> {
		return fetch(`${this.#base}/${path}`, { ...init, signal: this.#abort.signal, headers: { 'content-type': 'application/json' } });
	}

	async #loadAll() {
		const response = await this.#api('snapshot');
		if (!response.ok) throw new Error(m.device_load_failed({ status: response.status }));
		const snapshot = (await response.json()) as { files_version: number; files: { path: string; content: string; encoding?: 'base64'; updated_at: string }[] };
		await this.#wc!.mount(toFileTree(snapshot.files));
		this.#known = new Map(snapshot.files.map((file) => [file.path, file.updated_at]));
		this.#version = snapshot.files_version;
		this.#seen = snapshot.files_version;
	}

	async #write(path: string, content: string | Uint8Array) {
		const dir = parentDir(path);
		if (dir) await this.#wc!.fs.mkdir(dir, { recursive: true });
		await this.#wc!.fs.writeFile(path, content);
	}

	/** Copies in what changed since the last sync; returns the paths that changed. */
	async #sync(): Promise<string[]> {
		const response = await this.#api('manifest');
		if (!response.ok) return [];
		const manifest = (await response.json()) as { files_version: number; files: ManifestEntry[] };
		const { changed, removed } = diffManifest(this.#known, manifest.files);
		for (const path of changed) {
			const file = await this.#api(`files/${path}`);
			if (file.ok) await this.#write(path, new Uint8Array(await file.arrayBuffer()));
		}
		for (const path of removed) {
			await this.#wc!.fs.rm(path, { force: true });
			this.#known.delete(path);
		}
		for (const file of manifest.files) this.#known.set(file.path, file.updated_at);
		this.#version = manifest.files_version;
		return [...changed, ...removed];
	}

	async #run(command: string, args: string[], watch: { quietMs?: number; settled?: () => Promise<boolean>; timeoutMs?: number } = {}): Promise<{ code: number | null; output: string; hung?: boolean }> {
		this.#append(`\n$ ${[command, ...args].join(' ')}\n`);
		const proc = await this.#wc!.spawn(command, args, { env: this.#env });
		let output = '';
		let lastOutputAt = Date.now();
		void proc.output.pipeTo(
			new WritableStream({
				write: (chunk) => {
					output += chunk;
					lastOutputAt = Date.now();
					this.#append(chunk);
				},
			}),
		);
		const result = await watchExit(proc.exit, { lastOutputAt: () => lastOutputAt, settled: watch.settled ?? (async () => false), quietMs: watch.quietMs ?? COMMAND_TIMEOUT_MS, timeoutMs: watch.timeoutMs ?? COMMAND_TIMEOUT_MS });
		if (result === 'timeout' || result === 'hung') proc.kill();
		if (result === 'timeout') return { code: null, output: `${output}\n(stopped: took too long)` };
		if (result === 'hung') return { code: null, output, hung: true };
		return { code: result, output };
	}

	async #installed(): Promise<boolean> {
		try {
			return (await this.#wc!.fs.readdir('node_modules')).length > 0;
		} catch {
			return false;
		}
	}

	async #loadRelayAccess(): Promise<boolean> {
		const response = await this.#api('relay/preview').catch(() => null);
		if (!response?.ok) return response?.status === 404;
		this.#env = previewEnv(PREVIEW_ENV, (await response.json()) as RelayPreviewAccess);
		return true;
	}

	async #check(ok: boolean, output: string) {
		if (this.#checked) return;
		this.#checked = true;
		await this.#api('runtime/check', { method: 'POST', body: JSON.stringify({ files_version: this.#version, ok, output: output.slice(-OUTPUT_TAIL) }) }).catch(() => null);
	}

	/** Installs and starts the dev server once there's a package.json; until then it waits for Koda's first build. */
	async #prepareIfReady() {
		const packageJson = await this.#wc!.fs.readFile('package.json', 'utf-8').catch(() => undefined);
		if (!packageJson) {
			this.phase = 'building';
			return;
		}
		if (!this.#relayLoaded) this.#relayLoaded = await this.#loadRelayAccess();
		this.phase = 'starting';
		const installWatch = { quietMs: INSTALL_QUIET_MS, settled: () => this.#installed() };
		let install = await this.#run('npm', ['install'], installWatch);
		if (needsForcedInstall(install.code, install.output)) {
			install = await this.#run('npm', ['install', '--force'], installWatch);
		}
		const installed = install.code === 0 || install.hung === true;
		await this.#check(installed, install.output);
		if (!installed) return this.#fail(m.device_install_failed());
		await this.#startDev();
	}

	async #startDev() {
		const previous = this.#dev;
		this.#dev = null;
		previous?.kill();
		this.previewUrl = null;
		this.phase = 'starting';
		this.#append('\n$ npm run dev\n');
		const proc = await this.#wc!.spawn('npm', ['run', 'dev'], { env: this.#env });
		this.#dev = proc;
		void proc.output.pipeTo(new WritableStream({ write: (chunk) => this.#append(chunk) }));
		void proc.exit.then((code) => {
			if (this.#stopped || this.#dev !== proc) return;
			this.#fail(m.device_dev_stopped({ code }));
		});
	}

	async #runForKoda(run: ClaimedRun) {
		const install = run.command === 'npm' && (run.args[0] === 'install' || run.args[0] === 'i');
		const timeoutMs = run.timeout_seconds ? Math.max(RUN_MIN_MS, run.timeout_seconds * MS_PER_SECOND - RUN_MARGIN_MS) : COMMAND_TIMEOUT_MS;
		const watch = install ? { quietMs: INSTALL_QUIET_MS, settled: () => this.#installed(), timeoutMs } : { timeoutMs };
		let outcome = await this.#run(run.command, run.args, watch).catch((problem) => ({ code: null as number | null, output: String(problem), hung: false }));
		if (install && needsForcedInstall(outcome.code, outcome.output, run.args)) {
			outcome = await this.#run(run.command, [...run.args, '--force'], watch).catch((problem) => ({ code: null as number | null, output: String(problem), hung: false }));
		}
		const result = outcome.hung ? { code: 0, output: outcome.output } : outcome;
		await this.#api(`runtime/runs/${run.id}`, { method: 'POST', body: JSON.stringify({ exit_code: result.code, output: result.output.slice(-OUTPUT_TAIL) }) }).catch(() => null);
		// New packages need the dev server restarted to be picked up.
		if (install) await this.#startDev();
	}

	/** Checks in (keeping the project marked open here), queueing syncs and Koda's commands. */
	async #loop() {
		while (!this.#stopped) {
			try {
				const response = await this.#api(`runtime?since=${this.#seen}&wait=${CHECK_IN_WAIT_SECS}`);
				if (!response.ok) {
					await sleep(RETRY_MS);
					continue;
				}
				const { files_version, runs } = (await response.json()) as { files_version: number; runs: ClaimedRun[] };
				if (files_version !== this.#seen) {
					this.#seen = files_version;
					void this.#serial(async () => {
						if (this.#version === this.#seen) return;
						const changed = await this.#sync();
						if (this.phase === 'building' || changed.includes('package.json')) await this.#prepareIfReady();
					}).catch((problem) => this.#append(`\n(${String(problem)})\n`));
				}
				for (const run of runs) void this.#serial(() => this.#runForKoda(run));
			} catch {
				if (this.#stopped) return;
				await sleep(RETRY_MS);
			}
		}
	}
}
