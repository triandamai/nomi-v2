import { m } from '$lib/paraglide/messages';
import { projectApi } from '$lib/projectApi';
import { ProjectRuntime } from '$lib/projectRuntime.svelte';
import { SandboxRuntime, type SandboxStatus } from '$lib/sandboxRuntime.svelte';

/** What the page needs from either runtime. */
export type AnyRuntime = ProjectRuntime | SandboxRuntime;

/**
 * The sandbox when the project runs there and it's available; otherwise the browser's
 * WebContainer, which is also the fallback when the sandbox's status can't be read.
 */
export async function createRuntime(projectId: string): Promise<AnyRuntime> {
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
