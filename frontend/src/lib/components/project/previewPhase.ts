import { m } from '$lib/paraglide/messages';
import type { RuntimePhase } from '$lib/projectRuntime.svelte';

/** What the in-browser dev server is doing, in words: shown by the preview and its address bar. */
export const PHASE_LABEL: Record<RuntimePhase, () => string> = {
	unsupported: m.project_phase_unsupported,
	booting: m.project_phase_booting,
	loading: m.project_phase_loading,
	waiting: m.project_phase_waiting,
	installing: m.project_phase_installing,
	checking: m.project_phase_checking,
	starting: m.project_phase_starting,
	running: m.project_phase_running,
	failed: m.project_phase_failed,
	stopped: m.project_phase_stopped,
};
