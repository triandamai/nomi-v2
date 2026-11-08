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

/** Runs this project in the browser instead of the sandbox for the rest of this page session. */
export function forceBrowserRuntime(projectId: string) {
	forced.add(projectId);
	try {
		sessionStorage.setItem(FORCED_KEY, JSON.stringify([...new Set([...storedForced(), projectId])]));
	} catch {
		// The in-memory choice still holds.
	}
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
