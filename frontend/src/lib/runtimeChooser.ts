import { m } from '$lib/paraglide/messages';
import { projectApi } from '$lib/projectApi';
import { ProjectRuntime } from '$lib/projectRuntime.svelte';
import { SandboxRuntime, type SandboxStatus } from '$lib/sandboxRuntime.svelte';

/** What the page needs from either runtime. */
export type AnyRuntime = ProjectRuntime | SandboxRuntime;

const FORCED_KEY = 'nomi.browserRuntime';
const forced = new Set<string>();

function storedForced(): string[] {
	try {
		return JSON.parse(sessionStorage.getItem(FORCED_KEY) ?? '[]') as string[];
	} catch {
		return [];
	}
}

/**
 * Runs this project in the browser instead of the sandbox for the rest of this page session, and
 * asks the backend to stop the sandbox so it doesn't keep running (and billing) beside the browser.
 */
export function forceBrowserRuntime(projectId: string) {
	forced.add(projectId);
	try {
		sessionStorage.setItem(FORCED_KEY, JSON.stringify([...new Set([...storedForced(), projectId])]));
	} catch {
		// The in-memory choice still holds.
	}
	// Best effort: a sandbox that can't be stopped now idles out on its own.
	void projectApi(projectId, 'sandbox/stop', { method: 'POST' }).catch(() => undefined);
}

export function browserRuntimeForced(projectId: string): boolean {
	return forced.has(projectId) || storedForced().includes(projectId);
}

/**
 * The sandbox when the project runs there and it's available; otherwise the browser's
 * WebContainer, which is also the fallback when the sandbox's status can't be read.
 */
export async function createRuntime(projectId: string): Promise<AnyRuntime> {
	if (browserRuntimeForced(projectId)) return new ProjectRuntime(projectId);
	let status: SandboxStatus | null = null;
	try {
		status = await projectApi<SandboxStatus>(projectId, 'sandbox');
	} catch {
		// Not a sandbox project, or the call failed: the browser runtime is the default.
	}
	if (status?.runtime === 'sandbox' && status.available) return new SandboxRuntime(projectId);
	const runtime = new ProjectRuntime(projectId);
	if (status?.runtime === 'sandbox') runtime.log = `${m.project_sandbox_unavailable_fallback()}\n`;
	return runtime;
}

/** How often the page asks whether there is a build to preview yet. */
export const READY_POLL_MS = 5_000;

/**
 * Whether the project has a build to preview. A project the crew is still planning, designing or
 * building for the first time has none; one whose status can't be read previews as it always did.
 */
export async function previewReady(projectId: string): Promise<boolean> {
	try {
		const status = await projectApi<SandboxStatus>(projectId, 'sandbox');
		return status?.preview_ready !== false;
	} catch {
		return true;
	}
}

/**
 * Calls `start` once there is a build to preview: at once when there already is one, otherwise
 * when Koda's first build is done. `onwaiting` hears when the wait begins and ends. Returns what
 * stops the wait.
 */
export function startWhenReady(projectId: string, start: () => void, onwaiting: (waiting: boolean) => void): () => void {
	let cancelled = false;
	let waiting = false;
	let timer: ReturnType<typeof setTimeout> | null = null;
	const ask = async () => {
		const ready = await previewReady(projectId);
		if (cancelled) return;
		if (ready) {
			if (waiting) onwaiting(false);
			start();
			return;
		}
		if (!waiting) {
			waiting = true;
			onwaiting(true);
		}
		timer = setTimeout(() => void ask(), READY_POLL_MS);
	};
	void ask();
	return () => {
		cancelled = true;
		if (timer) clearTimeout(timer);
	};
}
