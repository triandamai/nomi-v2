import type { DeployOverview } from '$lib/adapters/api';
import type { PublishState } from '$lib/projects/deploy';
import type { TreeEntry } from '$lib/projects/code';
import type { PhonePill } from '$lib/ds/projects/PhoneWorkspaceNav.svelte';

// The phone workspace's own reading of the shared state (Mobile Build contract, docs/ux/work/
// phone-build/40-ux-contract.md): the publish pill per role (§2), the failed publish's card (G3,
// G8), the publish that went live while it was open, and Code's tree as phone rows.

/** One sheet at a time on a phone: the desktop's tools, the confirms and the phone's own. */
export type PhoneSheet =
	| 'tools'
	| 'publish'
	| 'mine'
	| 'deployments'
	| 'domain'
	| 'monitor'
	| 'database'
	| 'settings'
	| 'visibility'
	| 'people'
	| 'access'
	| 'invite'
	| 'environment'
	| 'transfer'
	| 'takedown'
	| 'delete'
	| 'share'
	| 'export'
	| 'github'
	| 'presence';

type PillActions = { onpublish: () => void; onask: () => void; onopen: () => void; asking: boolean };

/**
 * The pill by the deploy overview's state and the person's role: an admin publishes and retries; a
 * member asks (also where this Nomi can't publish, so the admins still hear), then waits on their
 * request; a viewer sees the state and nothing to press (the backend refuses them a publish, F1).
 */
export function publishPill(button: PublishState, role: string | null, asked: boolean, actions: PillActions): PhonePill {
	if (button.state === 'publishing') return { kind: 'publishing', elapsed: button.elapsed };
	if (button.state === 'idle') return { kind: 'idle' };
	if (role === 'viewer') return button.state === 'failed' ? { kind: 'failed' } : { kind: 'none' };
	if (role === 'member') return asked ? { kind: 'asked', onopen: actions.onopen } : { kind: 'ask', busy: actions.asking, onask: actions.onask };
	if (button.state === 'ready') return { kind: 'ready', count: button.count, onpublish: actions.onpublish };
	if (button.state === 'failed') return { kind: 'failed', onretry: actions.onpublish };
	return { kind: 'none' };
}

export type FailedPublish = { number: number; step: string; message: string; tail: string[]; elapsed: string; live: number | null };

const elapsed = (from: string, to: string | null) => {
	const seconds = to ? Math.max(0, Math.round((Date.parse(to) - Date.parse(from)) / 1000)) : 0;
	return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
};

/**
 * The newest production publish when it failed: where it stopped and what it said (the backend's
 * `failure`, else the row's error), its log's last lines, and the deploy still live (null when
 * nothing is: the first publish failed, G3). `_timeZone` is kept for the card's times later.
 */
export function failedPublish(overview: DeployOverview, _timeZone?: string): FailedPublish | null {
	const production = overview.publishes.filter((one) => one.environment === 'production');
	const [newest] = production;
	if (!newest || newest.status !== 'failed') return null;
	const live = production.find((one) => one.is_live);
	const failure = newest.failure ?? null;
	return {
		number: newest.version,
		step: failure?.step || 'publish',
		message: failure?.message || newest.error || '',
		tail: failure?.tail ?? [],
		elapsed: elapsed(newest.created_at, newest.finished_at),
		live: live?.version ?? null,
	};
}

/** The publish that was running before and is live now: it went live while the workspace was open. */
export function settledLive(before: DeployOverview | null, after: DeployOverview | null): string | null {
	const running = before?.publishes.find((one) => one.status === 'running');
	if (!running || !after) return null;
	const now = after.publishes.find((one) => one.id === running.id);
	return now?.status === 'succeeded' ? now.id : null;
}

export type PhoneTreeRow = { kind: TreeEntry['kind']; path: string; label: string; meta: string | undefined; open: boolean };

/**
 * Code's tree as phone rows: the top level, and inside each folder that's open, its entries (named
 * by their path from the top, since a phone row has no room to indent). A folder says what it holds.
 */
export function phoneTree(entries: TreeEntry[], open: string[]): PhoneTreeRow[] {
	const shown = (entry: TreeEntry) => {
		const parts = entry.path.split('/');
		return parts.slice(0, -1).every((_, at) => open.includes(parts.slice(0, at + 1).join('/')));
	};
	return entries.filter(shown).map((entry) => {
		const children = entries.filter((one) => one.depth === entry.depth + 1 && one.path.startsWith(`${entry.path}/`));
		return {
			kind: entry.kind,
			path: entry.path,
			label: entry.path,
			meta: entry.kind === 'folder' ? children.map((one) => one.name).join(' · ') : undefined,
			open: entry.kind === 'folder' && open.includes(entry.path),
		};
	});
}
