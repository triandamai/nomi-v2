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

export class SandboxRuntime {
	phase = $state<SandboxPhase>('loading');
	previewUrl = $state<string | null>(null);
	log = $state('');
	error = $state<string | null>(null);

	#base: string;
	#started = false;
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
		// Opening the workspace wakes a sandbox that's asleep or not made yet.
		if (status && (status.phase === 'idle' || status.phase === 'stopped' || status.state === 'none')) {
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
		this.phase = view.phase;
		this.previewUrl = view.previewUrl;
		this.error = view.error;
		if (status.phase === 'waking' || status.phase === 'running' || status.phase === 'error') await this.#readLog();
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
