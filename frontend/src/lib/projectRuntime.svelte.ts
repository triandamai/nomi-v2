// Runs a project in the browser with WebContainer (StackBlitz): loads its files, installs and
// checks it, starts its dev server for the preview, copies in Koda's changes as they're saved,
// and runs the commands Koda asks for (run_command), sending the output back.
//
// The page must be cross-origin isolated (COOP/COEP headers, set for project pages in
// hooks.server.ts); without that, or on a browser WebContainer doesn't support, the phase is
// 'unsupported' and the page falls back to the static preview.

import { WebContainer, type WebContainerProcess } from '@webcontainer/api';
import { m } from '$lib/paraglide/messages';
import { capLog, cleanOutput, diffManifest, parentDir, scriptsOf, toFileTree, type ManifestEntry } from '$lib/projectRuntime';

export type RuntimePhase =
	| 'unsupported'
	| 'booting'
	| 'loading'
	| 'waiting'
	| 'installing'
	| 'checking'
	| 'starting'
	| 'running'
	| 'failed'
	| 'stopped';

interface ClaimedRun {
	id: string;
	command: string;
	args: string[];
}

const PREVIEW_ENV = { NOMI_PREVIEW: '1' };
const CHECK_IN_WAIT_SECS = 20;
const COMMAND_TIMEOUT_MS = 10 * 60 * 1000;
const RETRY_MS = 3000;
/** WebContainer loads from StackBlitz's servers; give up waiting after this long. */
const BOOT_TIMEOUT_MS = 60_000;

// One WebContainer per page: booting a second one needs the first torn down.
let booted: Promise<WebContainer> | null = null;

async function boot(): Promise<WebContainer> {
	booted ??= WebContainer.boot({ coep: 'credentialless', workdirName: 'project' });
	return booted;
}

async function teardown() {
	const current = booted;
	booted = null;
	if (current) (await current.catch(() => null))?.teardown();
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export class ProjectRuntime {
	phase = $state<RuntimePhase>('booting');
	previewUrl = $state<string | null>(null);
	log = $state('');
	error = $state<string | null>(null);
	/** The install-and-check on opening failed (and whether Koda took it). */
	checkFailed = $state(false);
	handedToKoda = $state(false);
	/** The command Koda is running right now, if any. */
	running = $state<string | null>(null);

	#projectId: string;
	#wc: WebContainer | null = null;
	#dev: WebContainerProcess | null = null;
	#known = new Map<string, string>();
	#version = -1;
	/** The newest files version the backend reported (syncing may still be catching up). */
	#seen = -1;
	/** Installs, syncs and commands run one at a time, while check-ins carry on. */
	#queue: Promise<unknown> = Promise.resolve();
	#stopped = false;
	#abort = new AbortController();
	#checked = false;

	constructor(projectId: string) {
		this.#projectId = projectId;
	}

	async start() {
		if (typeof window === 'undefined' || !window.crossOriginIsolated) {
			this.phase = 'unsupported';
			return;
		}
		try {
			this.phase = 'booting';
			const timedOut = sleep(BOOT_TIMEOUT_MS).then(() => {
				throw new Error(m.project_boot_timeout());
			});
			this.#wc = await Promise.race([boot(), timedOut]);
			this.#wc.on('server-ready', (_port, url) => {
				if (this.#stopped) return;
				this.previewUrl = url;
				this.phase = 'running';
			});
			this.#wc.on('error', (e) => this.#fail(e.message));
			this.phase = 'loading';
			await this.#loadAll();
			void this.#loop();
			await this.#serial(() => this.#prepareIfReady());
		} catch (e) {
			if (!this.#stopped) this.#fail(e instanceof Error ? e.message : String(e));
		}
	}

	async stop() {
		this.#stopped = true;
		this.#abort.abort();
		this.#dev?.kill();
		this.phase = 'stopped';
		await teardown();
	}

	/** A file the person saved in the editor (the backend has it too). */
	async writeFile(path: string, content: string) {
		if (!this.#wc) return;
		await this.#write(path, content);
	}

	/** Starts the dev server again (after a crash, or on request). */
	async restart() {
		if (this.#stopped) return;
		// It never booted: only a fresh page can try again.
		if (!this.#wc) {
			location.reload();
			return;
		}
		this.error = null;
		await this.#startDev();
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
		this.log = capLog(this.log + cleanOutput(text));
	}

	async #api(path: string, init?: RequestInit): Promise<Response> {
		return fetch(`/projects/${this.#projectId}/api/${path}`, { ...init, signal: this.#abort.signal, headers: { 'content-type': 'application/json' } });
	}

	async #loadAll() {
		const response = await this.#api('snapshot');
		if (!response.ok) throw new Error(`couldn't load the project (${response.status})`);
		const snapshot = (await response.json()) as { files_version: number; files: { path: string; content: string; updated_at: string }[] };
		await this.#wc!.mount(toFileTree(snapshot.files));
		this.#known = new Map(snapshot.files.map((f) => [f.path, f.updated_at]));
		this.#version = snapshot.files_version;
		this.#seen = snapshot.files_version;
	}

	async #write(path: string, content: string) {
		const dir = parentDir(path);
		if (dir) await this.#wc!.fs.mkdir(dir, { recursive: true });
		await this.#wc!.fs.writeFile(path, content);
	}

	/** Copies in what changed since the last sync. Returns the paths that changed. */
	async #sync(): Promise<string[]> {
		const response = await this.#api('manifest');
		if (!response.ok) return [];
		const manifest = (await response.json()) as { files_version: number; files: ManifestEntry[] };
		const { changed, removed } = diffManifest(this.#known, manifest.files);
		for (const path of changed) {
			const file = await this.#api(`files/${path}`);
			if (!file.ok) continue;
			await this.#write(path, await file.text());
		}
		for (const path of removed) {
			await this.#wc!.fs.rm(path, { force: true });
			this.#known.delete(path);
		}
		for (const f of manifest.files) this.#known.set(f.path, f.updated_at);
		this.#version = manifest.files_version;
		return [...changed, ...removed];
	}

	async #run(command: string, args: string[]): Promise<{ code: number | null; output: string }> {
		const line = [command, ...args].join(' ');
		this.#append(`\n$ ${line}\n`);
		const proc = await this.#wc!.spawn(command, args, { env: PREVIEW_ENV });
		let output = '';
		void proc.output.pipeTo(
			new WritableStream({
				write: (chunk) => {
					output += chunk;
					this.#append(chunk);
				},
			}),
		);
		const timeout = sleep(COMMAND_TIMEOUT_MS).then(() => 'timeout' as const);
		const result = await Promise.race([proc.exit, timeout]);
		if (result === 'timeout') {
			proc.kill();
			this.#append(`\n(stopped after ${COMMAND_TIMEOUT_MS / 60000} minutes)\n`);
			return { code: null, output: cleanOutput(output) + '\n(stopped: took too long)' };
		}
		return { code: result, output: cleanOutput(output) };
	}

	async #packageJson(): Promise<string | undefined> {
		try {
			return await this.#wc!.fs.readFile('package.json', 'utf-8');
		} catch {
			return undefined;
		}
	}

	/** Installs, checks (once per opening, reported to Nomi) and starts the dev server, once the project has a package.json. */
	async #prepareIfReady() {
		const packageJson = await this.#packageJson();
		if (!packageJson) {
			this.phase = 'waiting';
			return;
		}
		this.phase = 'installing';
		const install = await this.#run('npm', ['install']);
		let ok = install.code === 0;
		let output = install.output;
		if (ok && scriptsOf(packageJson).check) {
			this.phase = 'checking';
			const check = await this.#run('npm', ['run', 'check']);
			ok = check.code === 0;
			output += `\n$ npm run check\n${check.output}`;
		}
		if (!this.#checked) {
			this.#checked = true;
			this.checkFailed = !ok;
			const response = await this.#api('runtime/check', {
				method: 'POST',
				body: JSON.stringify({ files_version: this.#version, ok, output: output.slice(-60_000) }),
			}).catch(() => null);
			if (response?.ok) this.handedToKoda = ((await response.json()) as { handed_to_koda: boolean }).handed_to_koda;
		}
		if (install.code !== 0) {
			this.#fail('npm install failed. See the terminal.');
			return;
		}
		await this.#startDev();
	}

	async #startDev() {
		const previous = this.#dev;
		this.#dev = null;
		previous?.kill();
		this.previewUrl = null;
		this.phase = 'starting';
		this.#append('\n$ npm run dev\n');
		const proc = await this.#wc!.spawn('npm', ['run', 'dev'], { env: PREVIEW_ENV });
		this.#dev = proc;
		void proc.output.pipeTo(new WritableStream({ write: (chunk) => this.#append(chunk) }));
		void proc.exit.then((code) => {
			if (this.#stopped || this.#dev !== proc) return;
			this.#fail(`The dev server stopped (exit code ${code}). See the terminal.`);
		});
	}

	async #runForKoda(run: ClaimedRun) {
		this.running = [run.command, ...run.args].join(' ');
		const result = await this.#run(run.command, run.args).catch((e) => ({ code: null, output: String(e) }));
		this.running = null;
		await this.#api(`runtime/runs/${run.id}`, { method: 'POST', body: JSON.stringify({ exit_code: result.code, output: result.output }) }).catch(() => null);
		// New packages need the dev server restarted to be picked up.
		if (run.command === 'npm' && (run.args[0] === 'install' || run.args[0] === 'i') && this.#dev) await this.#startDev();
	}

	/** Checks in (keeping the project marked open), queueing syncs and Koda's commands. */
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
						if (this.phase === 'waiting' || changed.includes('package.json')) await this.#prepareIfReady();
					}).catch((e) => this.#append(`\n(couldn't copy in the latest files: ${e instanceof Error ? e.message : e})\n`));
				}
				for (const run of runs) void this.#serial(() => this.#runForKoda(run));
			} catch (e) {
				if (this.#stopped) return;
				this.#append(`\n(lost touch with Nomi: ${e instanceof Error ? e.message : e}; retrying)\n`);
				await sleep(RETRY_MS);
			}
		}
	}
}
