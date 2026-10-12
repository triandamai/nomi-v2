// A project's publishing state, shared by the project's top bar (the Publish button and the
// live address) and its Deployments sheet. Polls while a publish runs.

import { projectApi } from '$lib/projectApi';
import type { DeployOverview, ProjectPublish } from '$lib/types';

const POLL_MS = 3000;

export class ProjectDeploy {
	overview = $state<DeployOverview | null>(null);
	error = $state<string | null>(null);
	busy = $state(false);
	readonly projectId: string;
	#timer: ReturnType<typeof setTimeout> | null = null;
	#stopped = false;

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
		if (this.#stopped || !this.running) return;
		this.#timer = setTimeout(() => void this.#poll(), POLL_MS);
	}

	// While running, asking for the newest publish makes the backend check with Shipyard too.
	async #poll() {
		const running = this.overview?.publishes.find((p) => p.status === 'running');
		if (running && !document.hidden) {
			await projectApi(this.projectId, `deploy/publishes/${running.id}`).catch(() => null);
		}
		await this.refresh();
	}

	async publish(): Promise<boolean> {
		return this.#act(() => projectApi(this.projectId, 'deploy/publish', { method: 'POST', body: {} }));
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

	stop() {
		this.#stopped = true;
		if (this.#timer) clearTimeout(this.#timer);
	}
}
