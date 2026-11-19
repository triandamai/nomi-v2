import { describe, expect, it } from 'vitest';
import type { ProjectSummary } from '$lib/adapters/api';
import { alertTitle, databaseName, detailLine, liveLine, moveTargets, panelParts, peopleLine, projectChats, took } from './about';
import { projectCards } from './list';

const now = new Date('2026-11-13T10:00:00Z');
const summary = (extra: Partial<ProjectSummary> = {}): ProjectSummary => ({
	id: 'p1',
	session_id: 's1',
	name: 'salah-daily-tracker',
	description: null,
	status: 'ready',
	created_at: '2026-11-01T00:00:00Z',
	updated_at: '2026-11-11T10:00:00Z',
	created_by: 'Trian',
	open_issues: 0,
	deploys_count: 14,
	deploy_url: 'https://salah-daily-tracker-0ab8f8.trian.space',
	...extra,
});
const [live] = projectCards([summary()], now);
const [draft] = projectCards([summary({ deploy_url: null, deploys_count: 0 })], now);
const about = {
	files: 23,
	stack: 'sveltekit',
	database: 'none',
	people: [
		{ name: 'Mario', owner: false, me: true },
		{ name: 'Trian', owner: true, me: false },
	],
	deploys: { count: 14, builtIn: 41, liveNumber: 14, liveAt: '2026-11-11T09:00:00Z', failedNumber: null },
};

describe('what’s in a project', () => {
	it('says each part as the panel shows it', () => {
		expect(panelParts(live, about)).toEqual({
			code: '23 files · SvelteKit',
			url: 'salah-daily-tracker-0ab8f8.trian.space',
			data: 'none yet',
			deploys: '14 · last built in 41s',
			members: 'Trian (owner), you',
		});
		expect(panelParts(draft, { ...about, files: 1, stack: 'static', database: 'postgres', deploys: { count: 0, builtIn: null, liveNumber: null, liveAt: null, failedNumber: null } })).toMatchObject({
			code: '1 file · static site',
			url: 'after the first deploy',
			data: 'Postgres',
			deploys: 'none yet',
		});
	});

	it('says “checking…” while it’s read, and that a part couldn’t be read, never 0', () => {
		expect(panelParts(live, undefined)).toMatchObject({ code: 'checking…', data: 'checking…', deploys: 'checking…', members: 'checking…', url: 'salah-daily-tracker-0ab8f8.trian.space' });
		expect(panelParts(live, null).data).toBe('couldn’t be read');
		expect(panelParts(live, { ...about, database: null, people: null }).data).toBe('couldn’t be read');
		expect([databaseName('none'), databaseName('sqlite'), databaseName('mysql')]).toEqual([null, 'SQLite', 'mysql']);
	});

	it('names who can open it, the owner first, at most three', () => {
		expect(peopleLine([{ name: 'Mario', owner: true, me: true }])).toBe('you (owner)');
		expect(
			peopleLine([
				{ name: 'A', owner: false, me: false },
				{ name: 'B', owner: false, me: false },
				{ name: 'C', owner: true, me: false },
				{ name: 'D', owner: false, me: false },
				{ name: 'E', owner: false, me: false },
			]),
		).toBe('C (owner), A, B +2');
	});

	it('times a build in seconds, then minutes', () => {
		expect([took(41), took(65), took(600)]).toEqual(['41s', '1m 05s', '10m 00s']);
	});

	it('says which deploy is live and since when, only once there is an address', () => {
		expect(liveLine(live, about, now)).toBe('live · deploy #14 · 2d ago');
		expect(liveLine(live, undefined, now)).toBe('live · deploy #14');
		expect(liveLine(draft, about, now)).toBeUndefined();
	});

	it('never says “deploy #0”: a live deploy without a number falls back to the card’s, or leaves it out', () => {
		const zero = { ...about, deploys: { ...about.deploys, liveNumber: 0 } };
		expect(liveLine(live, zero, now)).toBe('live · deploy #14 · 2d ago');
		const [first] = projectCards([summary({ deploys_count: 0 })], now);
		expect(liveLine(first, zero, now)).toBe('live · 2d ago');
	});

	it('says the state under a project’s name on a phone, as the Detail panel draws it (AL711)', () => {
		const [failed] = projectCards([summary({ last_deploy: { status: 'failed', at: '2026-11-13T09:00:00Z' } })], now);
		const broke = { ...about, deploys: { ...about.deploys, liveNumber: 5, failedNumber: 6 } };
		expect(detailLine(failed, broke, now)).toBe('deploy #6 failed · #5 still live');
		expect(alertTitle(failed, broke)).toBe('Deploy #6 failed');
		const [firstFailed] = projectCards([summary({ deploy_url: null, last_deploy: { status: 'failed', at: '2026-11-13T09:00:00Z' } })], now);
		expect(detailLine(firstFailed, { ...broke, deploys: { ...broke.deploys, liveNumber: null } }, now)).toBe('deploy #6 failed');
		// Before the deploys are read, the card's own words.
		expect(detailLine(failed, undefined, now)).toBe(failed.detail);
		expect(alertTitle(failed, undefined)).toBe('Deploy failed');
		const [changing] = projectCards([summary()], now, { active: new Set(['s1']) });
		expect(detailLine(changing, about, now)).toBe('Koda is changing it · #14 stays live');
		expect(detailLine(live, about, now)).toBe('live · deploy #14 · 2d ago');
		expect(detailLine(draft, about, now)).toBe(draft.footer);
	});
});

describe('a project’s chats', () => {
	it('lists its own chat (its workspace) and those moved into it, newest first', () => {
		const rows = [
			{ id: 's1', title: null, updated_at: '2026-11-11T10:00:00Z', project: 'p1', own: true, active: false },
			{ id: 'c2', title: 'Add Hijri offset setting', updated_at: '2026-11-13T09:00:00Z', project: 'p1', own: false, active: false },
			{ id: 'c3', title: 'Elsewhere', updated_at: '2026-11-13T09:00:00Z', project: 'p9', own: false, active: false },
		];
		expect(projectChats(rows, live, now)).toEqual([
			{ title: 'Add Hijri offset setting', age: '1h', href: '/chat/c2' },
			{ title: 'Building it with Koda', age: '2d', href: '/projects/p1' },
		]);
	});
});

describe('where a project can move', () => {
	it('shows where it is now, then each workspace with the person’s role and its people', () => {
		expect(
			moveTargets({
				current: { id: 'w1', name: 'Trian’s space', members: 3 },
				targets: [
					{ id: 'w2', name: 'Kopi Kita', members: 5, role: 'admin', owner: 'Sari' },
					{ id: 'w3', name: 'Solo', members: null, role: null, owner: null },
				],
			}),
		).toEqual([
			{ id: 'w1', name: 'Trian’s space', meta: 'current · 3 members', tone: 'nomi', members: 3, current: true },
			{ id: 'w2', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'tara', members: 5, owner: 'Sari' },
			{ id: 'w3', name: 'Solo', meta: '', tone: 'koda', members: 0 },
		]);
	});
});
