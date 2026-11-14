import { describe, expect, it } from 'vitest';
import { activityLine, deleteRefusal, lastActive } from './orgs';

const NOW = new Date('2026-11-14T12:00:00Z').getTime();
const ago = (minutes: number) => new Date(NOW - minutes * 60_000).toISOString();

describe('orgs', () => {
	it('names what happened, in which project, and who did it', () => {
		expect(activityLine(null, NOW)).toBeNull();
		const published = activityLine({ kind: 'published', project: 'warung-kopi', by: 'Ana', at: ago(180) }, NOW)!;
		expect(published).toContain('Ana');
		expect(published).toContain('warung-kopi');
		const worked = activityLine({ kind: 'worked_on', project: 'menu', by: null, at: ago(10) }, NOW)!;
		expect(worked).toContain('menu');
		expect(worked).not.toEqual(published);
	});

	it('says active now for someone seen in the last few minutes, else how long ago', () => {
		expect(lastActive(null, NOW)).toBeNull();
		expect(lastActive('not-a-date', NOW)).toBeNull();
		const now = lastActive(ago(2), NOW)!;
		const earlier = lastActive(ago(60 * 24 * 21), NOW)!;
		expect(now).toBeTruthy();
		expect(earlier).not.toEqual(now);
	});

	it('says why a workspace stays', () => {
		const projects = deleteRefusal('move or delete its projects first');
		const personal = deleteRefusal("a personal workspace can't be deleted");
		const only = deleteRefusal("it's the only workspace one of its members has");
		expect(new Set([projects, personal, only]).size).toBe(3);
	});
});
