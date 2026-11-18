import type { SandboxStatus } from '$lib/adapters/api';

// A project's sandbox (the backend's runtime for an app with a dev server): whether this project
// runs there, and what its preview is doing. The preview address carries a secret token: it only
// ever goes into the preview frame, never into text, a log or another request.

/** `building`: the crew hasn't finished a first build, so there is nothing to start yet. */
export type SandboxPhase = 'loading' | 'building' | 'waking' | 'starting' | 'running' | 'stopped' | 'failed';

/** The address if it's a real https URL, else null (never a `javascript:` or `data:` frame). */
export function safePreviewUrl(url: string | null): string | null {
	if (!url) return null;
	try {
		return new URL(url).protocol === 'https:' ? url : null;
	} catch {
		return null;
	}
}

/** The project runs in the sandbox, and it may be used now. */
export function usesSandbox(status: SandboxStatus | null): boolean {
	return status?.runtime === 'sandbox' && status.available;
}

/** The backend's status as the preview shows it; a running sandbox with no preview is starting, or failed when it says why. */
export function sandboxView(status: SandboxStatus): { phase: SandboxPhase; previewUrl: string | null; error: string | null } {
	const previewUrl = status.phase === 'running' ? safePreviewUrl(status.preview_url) : null;
	if (status.phase === 'error' || (status.phase === 'running' && !previewUrl && status.error)) return { phase: 'failed', previewUrl: null, error: status.error ?? '' };
	if (status.phase === 'running') return { phase: previewUrl ? 'running' : 'starting', previewUrl, error: null };
	if (status.phase === 'stopping' || status.phase === 'stopped') return { phase: 'stopped', previewUrl: null, error: null };
	if (status.preview_ready === false) return { phase: 'building', previewUrl: null, error: null };
	return { phase: 'waking', previewUrl: null, error: null };
}

// Colour and cursor codes a dev server prints, which mean nothing as text.
// eslint-disable-next-line no-control-regex
const TERMINAL_CODES = /\x1b\[[0-9;?]*[A-Za-z]/g;

/** The last line the sandbox printed, as what it's doing now; null before it printed anything. */
export function latestLine(log: string): string | null {
	const lines = log.replace(TERMINAL_CODES, '').split('\n').map((line) => line.trim()).filter(Boolean);
	return lines.at(-1) ?? null;
}
