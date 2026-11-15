import { describe, expect, it } from 'vitest';
import type { ProjectSummary } from '$lib/adapters/api';
import { filterCounts, hostOf, projectCards, runningFor, visibleCards } from './list';

const now = new Date('2026-11-13T10:00:00Z');
const project = (id: string, extra: Partial<ProjectSummary> = {}): ProjectSummary => ({
	id,
	session_id: `s-${id}`,
	name: id,
	description: null,
	status: 'ready',
	created_at: '2026-11-01T00:00:00Z',
	updated_at: '2026-11-11T10:00:00Z',
	created_by: 'Mario',
	open_issues: 0,
	chats_count: 1,
	deploys_count: 0,
	...extra,
});

describe('the project cards', () => {
	it('reads the backend’s state: live once it has an address, stopped mid-build, or a draft', () => {
		const cards = projectCards(
			[
				project('warung', { deploy_url: 'https://warung.trian.space', deploys_count: 3, chats_count: 2, description: 'Menu and pre-order' }),
				project('salah', { status: 'building' }),
				project('invoice', { status: 'planning' }),
				project('done', { status: 'ready' }),
			],
			now,
		);
		expect(cards.map((card) => [card.id, card.status, card.next])).toEqual([
			['warung', 'live', null],
			['salah', 'stopped', 'resume'],
			['invoice', 'draft', null],
			['done', 'draft', null],
		]);
		expect(cards[0]).toMatchObject({ url: 'warung.trian.space', footer: '2 chats · deploy #3 · 2d ago', description: 'Menu and pre-order', href: '/projects/warung' });
		expect(cards[1].detail).toBe('nothing deployed · files kept');
		expect(cards[2]).toMatchObject({ footer: '1 chat · 2d ago', description: 'Started by Mario' });
		expect(projectCards([project('fresh', { updated_at: '2026-11-13T09:59:30Z' })], now)[0].footer).toBe('1 chat · just now');
	});

	it('shows Koda at work, a publish running, and Deploy only where this Nomi publishes', () => {
		const projects = [
			project('salah', { deploy_url: 'https://salah.trian.space/', deploys_count: 15 }),
			project('warung', { status: 'building' }),
			project('budget', { files_count: 3, deploys_count: 0 }),
			project('empty', { files_count: 0 }),
		];
		const active = new Set(['s-salah', 's-warung']);
		const working = projectCards(projects, now, { active, publishing: true });
		expect(working.map((card) => [card.status, card.statusLabel, card.detail, card.next])).toEqual([
			['building', 'Koda is changing it', 'deploy #15 stays live', 'stop'],
			['building', undefined, undefined, 'stop'],
			['draft', undefined, 'not deployed yet · 3 files', 'deploy'],
			['draft', undefined, 'not deployed yet', null],
		]);
		expect(projectCards(projects, now, { publishing: false })[2].next).toBeNull();
		const deploying = projectCards(projects, now, { deploying: { budget: '2026-11-13T09:59:36Z' }, publishing: true })[2];
		expect(deploying).toMatchObject({ status: 'building', statusLabel: 'Koda is deploying', meta: '00:24', detail: 'deploy #1 · building 3 files', next: null });
	});

	it('says when Koda waits on an answer, a live app is down, or the last deploy failed, each with its action', () => {
		const [asked, down, failedLive, failedFirst] = projectCards(
			[
				project('asked', { waiting_on_you: 2 }),
				project('down', { deploy_url: 'https://down.trian.space', deploys_count: 4, live_status: 'down' }),
				project('fail', { deploy_url: 'https://fail.trian.space', deploys_count: 5, last_deploy: { status: 'failed', at: '2026-11-13T09:00:00Z' } }),
				project('first', { last_deploy: { status: 'failed', at: '2026-11-13T09:00:00Z' }, files_count: 3 }),
			],
			now,
		);
		expect(asked).toMatchObject({ status: 'attention', questions: 2, next: 'answer' });
		expect(down).toMatchObject({ status: 'down', url: 'down.trian.space', next: 'check' });
		expect(failedLive).toMatchObject({ status: 'failed', next: 'fix', detail: 'The last deploy failed · #5 is still live' });
		expect(failedFirst).toMatchObject({ status: 'failed', next: 'fix', detail: 'The first deploy failed' });
		expect(filterCounts([asked, down, failedLive, failedFirst])).toMatchObject({ attention: 3, live: 1 });
	});

	it('keeps Koda at work and a publish running ahead of what needs the person', () => {
		const [working, deploying] = projectCards(
			[project('w', { waiting_on_you: 1 }), project('d', { last_deploy: { status: 'failed', at: '2026-11-13T09:00:00Z' } })],
			now,
			{ active: new Set(['s-w']), deploying: { d: '2026-11-13T09:59:30Z' } },
		);
		expect([working.status, working.next, deploying.status]).toEqual(['building', 'stop', 'building']);
	});

	it('counts time running in minutes and seconds, and shows an address as its host', () => {
		expect(runningFor('2026-11-13T09:57:55Z', now)).toBe('02:05');
		expect(runningFor('2026-11-13T10:00:05Z', now)).toBe('00:00');
		expect(hostOf('https://warung.trian.space/')).toBe('warung.trian.space');
	});

	it('gives every project the same tint each time', () => {
		const [first] = projectCards([project('warung')], now);
		const [again] = projectCards([project('warung')], now);
		expect(first.tone).toBe(again.tone);
	});
});

describe('filtering, finding and sorting', () => {
	const cards = projectCards(
		[project('b-live', { deploy_url: 'https://b.trian.space', updated_at: '2026-11-10T00:00:00Z', created_at: '2026-10-01T00:00:00Z' }), project('a-build', { status: 'building', updated_at: '2026-11-12T00:00:00Z' }), project('c-draft', { status: 'planning', description: 'kopi menu' })],
		now,
	);

	it('counts each tab', () => {
		expect(filterCounts(cards)).toEqual({ all: 3, live: 1, building: 0, attention: 0 });
		expect(filterCounts(projectCards([project('a', { status: 'building' })], now, { active: new Set(['s-a']) })).building).toBe(1);
	});

	it('filters, finds by name or description, and sorts', () => {
		expect(visibleCards(cards, 'live', '', 'updated').map((card) => card.id)).toEqual(['b-live']);
		expect(visibleCards(cards, 'all', 'KOPI', 'updated').map((card) => card.id)).toEqual(['c-draft']);
		expect(visibleCards(cards, 'all', 'b.trian', 'updated').map((card) => card.id)).toEqual(['b-live']);
		expect(visibleCards(cards, 'all', '', 'name').map((card) => card.id)).toEqual(['a-build', 'b-live', 'c-draft']);
		expect(visibleCards(cards, 'all', '', 'updated').map((card) => card.id)).toEqual(['a-build', 'c-draft', 'b-live']);
		expect(visibleCards(cards, 'all', '', 'created').map((card) => card.id)[2]).toBe('b-live');
		expect(visibleCards(cards, 'all', '', 'status').map((card) => card.id)).toEqual(['b-live', 'a-build', 'c-draft']);
		expect(cards.find((card) => card.id === 'a-build')?.status).toBe('stopped');
	});
});
