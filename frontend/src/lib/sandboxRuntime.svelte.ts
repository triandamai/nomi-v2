// Runs a project in the backend's sandbox (boat.dev) instead of in the browser. The sandbox
// already has the project's files and runs Koda's commands itself, so this only starts it,
// keeps it awake with the page's heartbeat, follows its phase (live events, with a slow poll
// as a backstop) and shows its logs and preview.
//
// The preview address carries a secret token: it is only ever handed to the preview frame, never
// logged, shown as text or sent anywhere else.

import { projectApi } from '$lib/projectApi';
import { appendOutput, capLog, type ManifestEntry } from '$lib/projectRuntime';
import type { RuntimePhase } from '$lib/projectRuntime.svelte';

export type SandboxPhase = 'idle' | 'waking' | 'running' | 'stopping' | 'stopped' | 'error';

export interface SandboxStatus {
	runtime: string;
	available: boolean;
	state?: string;
	phase: SandboxPhase;
	preview_url: string | null;
	error: string | null;
}

/** The heartbeat that keeps a sandbox awake while the page is open. */
const HEARTBEAT_MS = 20_000;
/** The slow poll behind the live events (and the logs' tail). */
const POLL_MS = 5_000;
/**
 * The heartbeat's check-in. `runtime=sandbox` tells the backend this is the sandbox page, so it
 * keeps the sandbox awake (when the sandbox may be used) and leaves Koda's commands to it. The
 * browser runtime (the fallback, or "run in my browser") never sends it, and claims them itself.
 */
export const SANDBOX_CHECK_IN = 'runtime?runtime=sandbox';

/** The only scheme a preview address may have: it goes straight into an iframe's `src`. */
const PREVIEW_PROTOCOL = 'https:';

/** The address if it is a real https URL, else null (never a `javascript:` or `data:` frame). */
export function safePreviewUrl(url: string | null): string | null {
	if (!url) return null;
	try {
		return new URL(url).protocol === PREVIEW_PROTOCOL ? url : null;
	} catch {
		return null;
	}
}

/** The backend's phase in the runtime's words; a sandbox that runs but has no preview yet is still starting. */
export function toRuntimePhase(phase: SandboxPhase, previewUrl: string | null): RuntimePhase {
	switch (phase) {
		case 'waking':
			return 'waking';
		case 'running':
			return previewUrl ? 'running' : 'starting';
		case 'stopping':
			return 'stopped';
		case 'error':
			return 'failed';
		default:
			return 'waiting';
	}
}

export class SandboxRuntime {
	phase = $state<RuntimePhase>('loading');
	previewUrl = $state<string | null>(null);
	/** A sandbox preview is a public address, not a port inside the browser. */
	previewPort = $state<number | null>(null);
	log = $state('');
	/** The sandbox's files live on the backend; the page lists them from its own snapshot. */
	files = $state<ManifestEntry[]>([]);
	error = $state<string | null>(null);
	checkFailed = $state(false);
	handedToKoda = $state(false);
	running = $state<string | null>(null);

	#projectId: string;
	#stopped = false;
	#started = false;
	#offset = 0;
	#heartbeat: ReturnType<typeof setInterval> | null = null;
	#poll: ReturnType<typeof setInterval> | null = null;
	#unlisten: (() => void) | null = null;
	#fetching = false;

	constructor(projectId: string) {
		this.#projectId = projectId;
	}

	async start() {
		if (this.#heartbeat) return;
		this.#stopped = false;
		void this.#beat();
		this.#heartbeat = setInterval(() => void this.#beat(), HEARTBEAT_MS);
		this.#poll = setInterval(() => void this.#refresh(), POLL_MS);
		try {
			const status = await this.#refresh();
			// Opening the project wakes its sandbox if it is asleep.
			if (status && (status.phase === 'idle' || status.phase === 'stopped') && !this.#started) {
				this.#started = true;
				await this.#command('start');
			}
		} catch (e) {
			this.#fail(e instanceof Error ? e.message : String(e));
		}
	}

	async stop() {
		this.#stopped = true;
		if (this.#heartbeat) clearInterval(this.#heartbeat);
		if (this.#poll) clearInterval(this.#poll);
		this.#heartbeat = this.#poll = null;
		this.#unlisten?.();
		this.#unlisten = null;
		this.phase = 'stopped';
	}

	/** The sandbox's phase changes arrive on the project's live topic; `subscribe` is how the page listens to it. */
	listen(subscribe: (listener: (raw: string) => void) => () => void): () => void {
		this.#unlisten?.();
		const off = subscribe((raw) => {
			try {
				if (JSON.parse(raw).kind === 'SandboxChanged') void this.#refresh();
			} catch {
				// Not ours.
			}
		});
		this.#unlisten = off;
		return off;
	}

	async restart() {
		if (this.#stopped) return;
		this.error = null;
		try {
			await this.#command('stop');
			await this.#command('start');
		} catch (e) {
			this.#fail(e instanceof Error ? e.message : String(e));
		}
	}

	/** Asks the backend to start the sandbox again after it failed. */
	async retry() {
		if (this.#stopped) return;
		this.error = null;
		try {
			await this.#command('start');
		} catch (e) {
			this.#fail(e instanceof Error ? e.message : String(e));
		}
	}

	// The backend already has every file the person saves or deletes.
	async writeFile(_path: string, _content: string | Uint8Array) {}
	async removeFile(_path: string) {}
	async refreshRelay() {}

	async #command(action: 'start' | 'stop') {
		await projectApi(this.#projectId, `sandbox/${action}`, { method: 'POST' });
		await this.#refresh();
	}

	async #beat() {
		if (this.#stopped) return;
		try {
			await projectApi(this.#projectId, SANDBOX_CHECK_IN);
		} catch {
			// The next beat tries again.
		}
	}

	async #refresh(): Promise<SandboxStatus | null> {
		if (this.#stopped) return null;
		try {
			const status = await projectApi<SandboxStatus>(this.#projectId, 'sandbox');
			if (this.#stopped) return null;
			this.#apply(status);
			if (status.phase === 'waking' || status.phase === 'running' || status.phase === 'error') await this.#fetchLogs();
			return status;
		} catch {
			return null;
		}
	}

	#apply(status: SandboxStatus) {
		this.previewUrl = status.phase === 'running' ? safePreviewUrl(status.preview_url) : null;
		// A running sandbox whose install or dev server failed still runs Koda's commands; the page
		// shows why its preview is missing (and Retry tries the preview again).
		const previewFailed = status.phase === 'running' && !this.previewUrl && !!status.error;
		this.phase = previewFailed ? 'failed' : toRuntimePhase(status.phase, this.previewUrl);
		this.error = status.phase === 'error' || previewFailed ? (status.error ?? '') : null;
	}

	async #fetchLogs() {
		// A refresh that lands while another is still reading would read the same offset twice.
		if (this.#fetching) return;
		this.#fetching = true;
		try {
			const logs = await projectApi<{ text: string; next_offset: number }>(this.#projectId, `sandbox/logs?offset=${this.#offset}`);
			if (this.#stopped || !logs) return;
			if (logs.text) this.log = capLog(appendOutput(this.log, logs.text));
			this.#offset = logs.next_offset;
		} catch {
			// Logs are a nicety: the next poll tries again.
		} finally {
			this.#fetching = false;
		}
	}

	#fail(message: string) {
		this.error = message;
		this.phase = 'failed';
	}
}
