// A project's publishing state, shared by the project's top bar (the Publish button and the
// live address) and its Deployments sheet. Shipyard's webhooks reach the page over the
// project's socket (`/projects/{id}/ws`) and it reloads then; while a publish runs it also
// checks now and then, in case an event never arrives.

import { isDeployEvent } from '$lib/presenceView';
import { projectApi } from '$lib/projectApi';
import type { DeployOverview, ProjectPublish } from '$lib/types';

/** How often a running publish is checked with Shipyard when no event has come. */
const FALLBACK_MS = 30_000;
const MAX_RETRY_MS = 30_000;

export class ProjectDeploy {
	overview = $state<DeployOverview | null>(null);
	error = $state<string | null>(null);
	busy = $state(false);
	readonly projectId: string;
	/** Bumped on every update the socket brings, for views that show more (an open log). */
	changes = $state(0);
	#timer: ReturnType<typeof setTimeout> | null = null;
	#stopped = false;
	#socket: WebSocket | null = null;
	#retry: ReturnType<typeof setTimeout> | null = null;
	#retryMs = 1000;
	#listeners = new Set<(data: string) => void>();

	constructor(projectId: string) {
		this.projectId = projectId;
	}

	/** The newest publish, if any. */
	get latest(): ProjectPublish | null {
		return this.overview?.publishes[0] ?? null;
	}

	get running(): boolean {
		return this.overview?.publishes.some((p) => p.status === 'running') ?? false;
	}

	/** The app's own backend is being set up; the publish follows by itself when it's ready. */
	get preparing(): boolean {
		return this.overview?.backend_setup?.state === 'creating';
	}

	get isAdmin(): boolean {
		return this.overview?.role === 'admin';
	}

	async refresh(): Promise<void> {
		try {
			this.overview = await projectApi<DeployOverview>(this.projectId, 'deploy');
		} catch (e) {
			this.error = (e as Error).message;
		}
		this.#schedule();
	}

	#schedule() {
		if (this.#timer) clearTimeout(this.#timer);
		this.#timer = null;
		if (this.#stopped || !(this.running || this.preparing)) return;
		this.#timer = setTimeout(() => void this.#poll(), FALLBACK_MS);
	}

	/** Opens the project's socket; every message means "something changed, reload". */
	listen() {
		if (this.#stopped || this.#socket || typeof WebSocket === 'undefined') return;
		const socket = new WebSocket(`/projects/${this.projectId}/ws`);
		this.#socket = socket;
		socket.addEventListener('open', () => {
			// Whatever happened while it was closed came with no message.
			if (this.#retryMs > 1000) void this.refresh();
			this.#retryMs = 1000;
		});
		socket.addEventListener('message', (event: MessageEvent<string>) => {
			// The same topic carries presence and typing (see projectPresence): only publishing
			// changes reload the Deploy state.
			if (isDeployEvent(event.data)) {
				this.changes += 1;
				void this.refresh();
			}
			for (const listener of this.#listeners) listener(event.data);
		});
		socket.addEventListener('close', (event) => {
			this.#socket = null;
			// Signed out or no longer a member: the fallback check is all that's left.
			if (this.#stopped || event.code === 4401 || event.code === 4404) return;
			this.#retry = setTimeout(() => this.listen(), this.#retryMs + Math.random() * 250);
			this.#retryMs = Math.min(this.#retryMs * 2, MAX_RETRY_MS);
		});
	}

	// The fallback: asking for the running publish makes the backend check with Shipyard.
	async #poll() {
		const running = this.overview?.publishes.find((p) => p.status === 'running');
		if (running && !document.hidden) {
			await projectApi(this.projectId, `deploy/publishes/${running.id}`).catch(() => null);
		}
		await this.refresh();
	}

	/** Publishes the project's files to `environment` (production unless given). */
	async publish(environment?: string): Promise<boolean> {
		return this.#act(() => projectApi(this.projectId, 'deploy/publish', { method: 'POST', body: environment ? { environment } : {} }));
	}

	async rollback(publishId: string): Promise<boolean> {
		return this.#act(() => projectApi(this.projectId, `deploy/publishes/${publishId}/rollback`, { method: 'POST' }));
	}

	async takeOffline(environment = 'production'): Promise<boolean> {
		return this.#act(() => projectApi(this.projectId, `deploy/environments/${environment}`, { method: 'DELETE' }));
	}

	async #act(run: () => Promise<unknown>): Promise<boolean> {
		this.busy = true;
		this.error = null;
		try {
			await run();
			return true;
		} catch (e) {
			this.error = (e as Error).message;
			return false;
		} finally {
			this.busy = false;
			await this.refresh();
		}
	}

	/** Calls `listener` with every message on the project's live topic; returns the function that stops it. */
	onEvent(listener: (data: string) => void): () => void {
		this.#listeners.add(listener);
		return () => this.#listeners.delete(listener);
	}

	stop() {
		this.#stopped = true;
		if (this.#timer) clearTimeout(this.#timer);
		if (this.#retry) clearTimeout(this.#retry);
		this.#socket?.close();
		this.#socket = null;
	}
}
