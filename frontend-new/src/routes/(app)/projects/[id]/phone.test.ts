import { describe, expect, it, vi } from 'vitest';
import type { DeployOverview, PublishRow } from '$lib/adapters/api';
import type { TreeEntry } from '$lib/projects/code';
import { failedPublish, phoneTree, publishPill, settledLive } from './phone';

// The phone workspace's own logic (Mobile Build contract): the publish pill per role, the failed
// publish's card (G3, G8), the publish that went live while it was open, and Code's tree.

const row = (patch: Partial<PublishRow>): PublishRow => ({
	id: 'p1',
	environment: 'production',
	status: 'succeeded',
	url: null,
	error: null,
	version: 1,
	rollback_of: null,
	rollback_of_version: null,
	can_roll_back: false,
	created_by: 'Mario',
	started_by_agent: null,
	created_at: '2026-11-18T08:18:00Z',
	finished_at: '2026-11-18T08:18:41Z',
	is_live: false,
	...patch,
});
const overview = (publishes: PublishRow[], patch: Partial<DeployOverview> = {}): DeployOverview => ({ available: true, role: 'admin', url: null, publishes, backend_setup: null, unpublished_changes: 0, ...patch });
const actions = { onpublish: vi.fn(), onask: vi.fn(), onopen: vi.fn(), asking: false };

describe('the publish pill', () => {
	it('follows the state for an admin: up to date, ready with the count, publishing with its time, failed with retry', () => {
		expect(publishPill({ state: 'idle' }, 'admin', false, actions)).toEqual({ kind: 'idle' });
		expect(publishPill({ state: 'ready', count: 3 }, 'admin', false, actions)).toMatchObject({ kind: 'ready', count: 3 });
		expect(publishPill({ state: 'publishing', elapsed: '0:24' }, 'admin', false, actions)).toEqual({ kind: 'publishing', elapsed: '0:24' });
		const failed = publishPill({ state: 'failed' }, 'admin', false, actions);
		expect(failed).toMatchObject({ kind: 'failed' });
		expect(failed.kind === 'failed' && failed.onretry).toBe(actions.onpublish);
		expect(publishPill({ state: 'off' }, 'admin', false, actions)).toEqual({ kind: 'none' });
	});

	it('gives a member Ask to publish where Publish would be, also without Shipyard, then Asked', () => {
		expect(publishPill({ state: 'ready', count: 2 }, 'member', false, actions)).toMatchObject({ kind: 'ask' });
		expect(publishPill({ state: 'failed' }, 'member', false, actions)).toMatchObject({ kind: 'ask' });
		expect(publishPill({ state: 'off' }, 'member', false, actions)).toMatchObject({ kind: 'ask' });
		expect(publishPill({ state: 'ready', count: 2 }, 'member', true, actions)).toMatchObject({ kind: 'asked' });
		expect(publishPill({ state: 'publishing', elapsed: '0:05' }, 'member', true, actions)).toMatchObject({ kind: 'publishing' });
		expect(publishPill({ state: 'idle' }, 'member', false, actions)).toEqual({ kind: 'idle' });
	});

	it('gives a viewer the state only (F1): nothing to press, nothing while changes wait', () => {
		expect(publishPill({ state: 'ready', count: 2 }, 'viewer', false, actions)).toEqual({ kind: 'none' });
		expect(publishPill({ state: 'failed' }, 'viewer', false, actions)).toEqual({ kind: 'failed' });
		expect(publishPill({ state: 'idle' }, 'viewer', false, actions)).toEqual({ kind: 'idle' });
		expect(publishPill({ state: 'off' }, 'viewer', false, actions)).toEqual({ kind: 'none' });
	});
});

describe('a failed publish', () => {
	it('is the newest production publish, with the backend’s step, message and log, and the deploy still live', () => {
		const failed = row({ id: 'p2', version: 2, status: 'failed', error: 'build failed', created_at: '2026-11-18T08:45:00Z', finished_at: '2026-11-18T08:45:41Z', failure: { step: 'build', message: "Cannot find module './PickupSlots.svelte'", tail: ['vite v5', 'error'] } });
		const live = row({ id: 'p1', version: 1, is_live: true });
		expect(failedPublish(overview([failed, live]))).toEqual({ number: 2, step: 'build', message: "Cannot find module './PickupSlots.svelte'", tail: ['vite v5', 'error'], elapsed: '0:41', live: 1 });
	});

	it('says nothing is live when it was the first (G3), and falls back to the row’s error without a failure', () => {
		expect(failedPublish(overview([row({ status: 'failed', error: 'exit 1', failure: null })]))).toMatchObject({ number: 1, step: 'publish', message: 'exit 1', tail: [], live: null });
	});

	it('is gone once a newer publish runs or succeeded', () => {
		expect(failedPublish(overview([row({ id: 'p3', version: 3, status: 'running' }), row({ id: 'p2', version: 2, status: 'failed' })]))).toBeNull();
		expect(failedPublish(overview([row({ id: 'p1', is_live: true })]))).toBeNull();
	});
});

describe('the publish that went live', () => {
	it('is the one that was running and has succeeded since', () => {
		const running = overview([row({ id: 'p2', status: 'running', version: 2 })]);
		const done = overview([row({ id: 'p2', status: 'succeeded', version: 2, is_live: true })]);
		expect(settledLive(running, done)).toBe('p2');
		expect(settledLive(done, done)).toBeNull();
		expect(settledLive(null, done)).toBeNull();
		expect(settledLive(running, overview([row({ id: 'p2', status: 'failed', version: 2 })]))).toBeNull();
	});
});

describe('Code’s tree on a phone', () => {
	const entries: TreeEntry[] = [
		{ name: 'src', depth: 0, kind: 'folder', path: 'src' },
		{ name: 'lib', depth: 1, kind: 'folder', path: 'src/lib' },
		{ name: 'menu.ts', depth: 2, kind: 'file', path: 'src/lib/menu.ts' },
		{ name: 'app.html', depth: 1, kind: 'file', path: 'src/app.html' },
		{ name: 'README.md', depth: 0, kind: 'text', path: 'README.md' },
	];

	it('shows the top level with what each folder holds, and opens folders in place', () => {
		expect(phoneTree(entries, [])).toEqual([
			{ kind: 'folder', path: 'src', label: 'src', meta: 'lib · app.html', open: false },
			{ kind: 'text', path: 'README.md', label: 'README.md', meta: undefined, open: false },
		]);
		expect(phoneTree(entries, ['src']).map((one) => one.label)).toEqual(['src', 'src/lib', 'src/app.html', 'README.md']);
		expect(phoneTree(entries, ['src', 'src/lib']).map((one) => one.label)).toEqual(['src', 'src/lib', 'src/lib/menu.ts', 'src/app.html', 'README.md']);
	});
});
