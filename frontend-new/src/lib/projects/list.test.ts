import { describe, expect, it } from 'vitest';
import type { ProjectSummary } from '$lib/adapters/api';
import { filterCounts, projectCards, visibleCards } from './list';

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
	it('reads the backend’s state: live once it has an address, building, or a draft', () => {
		const cards = projectCards(
			[
				project('warung', { deploy_url: 'https://warung.trian.space', deploys_count: 3, chats_count: 2, description: 'Menu and pre-order' }),
				project('salah', { status: 'building' }),
				project('invoice', { status: 'planning' }),
				project('done', { status: 'ready' }),
			],
			now,
		);
		expect(cards.map((card) => [card.id, card.status])).toEqual([
			['warung', 'live'],
			['salah', 'building'],
			['invoice', 'draft'],
			['done', 'draft'],
		]);
		expect(cards[0]).toMatchObject({ url: 'https://warung.trian.space', footer: '2 chats · deploy #3 · 2d ago', description: 'Menu and pre-order', href: '/chat/s-warung' });
		expect(cards[2]).toMatchObject({ footer: '1 chat · 2d ago', description: 'Started by Mario' });
		expect(projectCards([project('fresh', { updated_at: '2026-11-13T09:59:30Z' })], now)[0].footer).toBe('1 chat · just now');
	});

	it('gives every project the same tint each time', () => {
		const [first] = projectCards([project('warung')], now);
		const [again] = projectCards([project('warung')], now);
		expect(first.tone).toBe(again.tone);
	});
});

describe('filtering, finding and sorting', () => {
	const cards = projectCards(
		[project('b-live', { deploy_url: 'https://b', updated_at: '2026-11-10T00:00:00Z', created_at: '2026-10-01T00:00:00Z' }), project('a-build', { status: 'building', updated_at: '2026-11-12T00:00:00Z' }), project('c-draft', { status: 'planning', description: 'kopi menu' })],
		now,
	);

	it('counts each tab', () => {
		expect(filterCounts(cards)).toEqual({ all: 3, live: 1, building: 1, attention: 0 });
	});

	it('filters, finds by name or description, and sorts', () => {
		expect(visibleCards(cards, 'live', '', 'updated').map((card) => card.id)).toEqual(['b-live']);
		expect(visibleCards(cards, 'all', 'KOPI', 'updated').map((card) => card.id)).toEqual(['c-draft']);
		expect(visibleCards(cards, 'all', 'https://b', 'updated').map((card) => card.id)).toEqual(['b-live']);
		expect(visibleCards(cards, 'all', '', 'name').map((card) => card.id)).toEqual(['a-build', 'b-live', 'c-draft']);
		expect(visibleCards(cards, 'all', '', 'updated').map((card) => card.id)).toEqual(['a-build', 'c-draft', 'b-live']);
		expect(visibleCards(cards, 'all', '', 'created').map((card) => card.id)[2]).toBe('b-live');
		expect(visibleCards(cards, 'all', '', 'status').map((card) => card.id)).toEqual(['b-live', 'a-build', 'c-draft']);
	});
});
