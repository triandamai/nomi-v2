import type { SandboxLogs, SandboxStatus } from '$lib/adapters/api';
import { sandboxView, type SandboxPhase } from './sandbox';
import { appendOutput, capLog } from './terminal';

// Runs a project in the backend's sandbox while its workspace is open: wakes it, keeps it awake
// with the page's heartbeat, follows its phase and reads its log. The sandbox already has the
// project's files and runs Koda's commands itself. Leaving the page stops the heartbeat; the
// sandbox then idles out on its own. The preview address holds a secret token, so it's kept in
// `previewUrl` for the preview frame and nowhere else.

/** The heartbeat that keeps the sandbox awake. */
const BEAT_MS = 20_000;
/** How often the phase and the log are read again. */
const POLL_MS = 5_000;
/** The backend's phases for a sandbox that isn't running or on its way up: opening the workspace wakes it. */
const AT_REST = ['idle', 'stopped', 'stopping'];

export class SandboxRuntime {
	phase = $state<SandboxPhase>('loading');
	previewUrl = $state<string | null>(null);
	log = $state('');
	error = $state<string | null>(null);

	#base: string;
	#started = false;
	/**
	 * A start was asked for and the backend hasn't moved yet: it still answers with where the
	 * sandbox is (stopped, stopping, not made) until its worker picks the start up.
	 */
	#asked = false;
	/** The crew hasn't finished a first build: the sandbox starts once it has. */
	#awaitingBuild = false;
	#offset = 0;
	#reading = false;
	#timers: ReturnType<typeof setInterval>[] = [];

	constructor(projectId: string) {
		this.#base = `/projects/${encodeURIComponent(projectId)}/sandbox`;
	}

	async start() {
		if (this.#started) return;
		this.#started = true;
		void this.#beat();
		this.#timers = [setInterval(() => void this.#beat(), BEAT_MS), setInterval(() => void this.#refresh(), POLL_MS)];
		const status = await this.#refresh();
		// Nothing to preview until the crew's first build: the reads below start it once there is.
		if (status?.preview_ready === false) {
			this.#awaitingBuild = true;
			return;
		}
		if (status) await this.#wake(status);
	}

	/**
	 * Opening the workspace wakes a sandbox that's asleep or not made yet. One that is still
	 * stopping (the workspace was closed a moment ago) starts again once it has stopped.
	 */
	async #wake(status: SandboxStatus) {
		if (AT_REST.includes(status.phase) || status.state === 'none') {
			this.phase = 'waking';
			await this.#command('start');
		}
	}

	stop() {
		this.#started = false;
		for (const timer of this.#timers) clearInterval(timer);
		this.#timers = [];
	}

	/** Starts the sandbox again after it failed. */
	async retry() {
		if (!this.#started) return;
		this.error = null;
		this.phase = 'waking';
		await this.#command('start');
	}

	async #command(action: 'start' | 'stop') {
		this.#asked = action === 'start';
		const response = await fetch(`${this.#base}/${action}`, { method: 'POST' }).catch(() => null);
		if (!this.#started) return;
		if (!response?.ok) {
			this.phase = 'failed';
			this.error = response ? (await response.text()).trim() : '';
			return;
		}
		await this.#refresh();
	}

	async #beat() {
		// A missed beat is caught by the next one.
		await fetch(`${this.#base}/beat`).catch(() => null);
	}

	async #refresh(): Promise<SandboxStatus | null> {
		const response = await fetch(`${this.#base}/status`).catch(() => null);
		if (!this.#started || !response?.ok) return null;
		const status = (await response.json()) as SandboxStatus;
		if (!this.#started) return null;
		const view = sandboxView(status);
		// The backend has moved on from where the sandbox rested: what it says from here on is shown as it is.
		if (!AT_REST.includes(status.phase)) this.#asked = false;
		this.phase = this.#asked ? 'waking' : view.phase;
		this.previewUrl = view.previewUrl;
		this.error = view.error;
		if (status.phase === 'waking' || status.phase === 'running' || status.phase === 'error') await this.#readLog();
		if (this.#awaitingBuild && status.preview_ready !== false) {
			this.#awaitingBuild = false;
			await this.#wake(status);
		}
		return status;
	}

	async #readLog() {
		// A poll that lands during a read would read the same offset twice.
		if (this.#reading) return;
		this.#reading = true;
		try {
			const response = await fetch(`${this.#base}/logs?offset=${this.#offset}`).catch(() => null);
			if (!this.#started || !response?.ok) return;
			const logs = (await response.json()) as SandboxLogs;
			if (logs.text) this.log = capLog(appendOutput(this.log, logs.text));
			this.#offset = logs.next_offset;
		} finally {
			this.#reading = false;
		}
	}
}
