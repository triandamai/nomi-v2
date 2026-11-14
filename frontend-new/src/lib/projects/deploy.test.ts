import { describe, expect, it } from 'vitest';
import type { DeployOverview, PublishRow } from '$lib/adapters/api';
import { deployRows, publishState, publishing } from './deploy';

const now = new Date('2026-11-14T08:45:00Z');
const row = (version: number, extra: Partial<PublishRow> = {}): PublishRow => ({
	id: `p${version}`,
	environment: 'production',
	status: 'succeeded',
	url: 'https://warung.trian.space',
	error: null,
	version,
	rollback_of: null,
	rollback_of_version: null,
	can_roll_back: true,
	created_by: 'Mario',
	started_by_agent: null,
	created_at: '2026-11-14T08:18:00Z',
	finished_at: '2026-11-14T08:19:00Z',
	is_live: false,
	...extra,
});
const overview = (extra: Partial<DeployOverview> = {}): DeployOverview => ({ available: true, role: 'owner', url: null, publishes: [], backend_setup: null, unpublished_changes: null, ...extra });

describe('the Publish button', () => {
	it('isn’t there where this Nomi can’t publish', () => {
		expect(publishState(overview({ available: false }), 3, now)).toEqual({ state: 'off' });
	});

	it('is ready with the files waiting, before the first deploy too', () => {
		expect(publishState(overview(), 4, now)).toEqual({ state: 'ready', count: 4 });
		expect(publishState(overview({ publishes: [row(1, { is_live: true })], unpublished_changes: 2 }), 9, now)).toEqual({ state: 'ready', count: 2 });
	});

	it('is up to date with nothing waiting, or nothing to publish yet', () => {
		expect(publishState(overview({ publishes: [row(1, { is_live: true })], unpublished_changes: 0 }), 9, now)).toEqual({ state: 'idle' });
		expect(publishState(overview(), 0, now)).toEqual({ state: 'idle' });
	});

	it('says how long a publish has been running, and that one failed', () => {
		expect(publishState(overview({ publishes: [row(2, { status: 'running', created_at: '2026-11-14T08:44:48Z', finished_at: null }), row(1, { is_live: true })] }), 9, now)).toEqual({ state: 'publishing', elapsed: '0:12' });
		expect(publishState(overview({ publishes: [row(2, { status: 'failed', error: 'build failed' }), row(1, { is_live: true })], unpublished_changes: 3 }), 9, now)).toEqual({ state: 'failed' });
	});

	it('knows when to keep watching', () => {
		expect(publishing(overview({ publishes: [row(2, { status: 'running' })] }))).toBe(true);
		expect(publishing(overview({ backend_setup: { message: 'Setting up' } }))).toBe(true);
		expect(publishing(overview({ publishes: [row(1)] }))).toBe(false);
	});
});

describe('the deployments', () => {
	it('names each, says when and who, and what can be done with it', () => {
		const rows = deployRows(
			[
				row(3, { rollback_of: 'p1', rollback_of_version: 1, is_live: true, created_at: '2026-11-14T08:44:00Z' }),
				row(2, { status: 'failed', error: 'Missing src/lib/menu.ts', created_by: null, started_by_agent: 'koda' }),
				row(1, { created_at: '2026-11-13T08:18:00Z' }),
			],
			now,
			'UTC',
		);
		expect(rows.map((one) => [one.number, one.title, one.state])).toEqual([
			['#3', 'Rolled back to #1', 'live'],
			['#2', 'Deploy #2', 'failed'],
			['#1', 'First deploy', 'rollback'],
		]);
		expect(deployRows([row(2, { is_live: true }), row(1, { can_roll_back: false })], now, 'UTC')[1].state).toBe('kept');
		expect(rows[0].meta).toBe('08:44 today · by Mario');
		expect(rows[1]).toMatchObject({ meta: '08:18 today · by Koda', error: 'Missing src/lib/menu.ts' });
		expect(rows[2].meta).toBe('08:18 yesterday · by Mario');
	});
});
