import { describe, expect, it } from 'vitest';
import type { MemoryItem } from '$lib/adapters/api';
import { memoryCounts, memoryTopics, strength } from './memory';

const memory = (id: string, kind: string, weight: number, extra: Partial<MemoryItem> = {}): MemoryItem => ({
	id,
	content: id,
	weight,
	kind,
	created_at: '2026-10-01T00:00:00Z',
	updated_at: '2026-10-10T00:00:00Z',
	uses: 0,
	last_used_at: null,
	needs_check: false,
	...extra,
});

const memories = [
	memory('Short, direct answers', 'preference', 4, { updated_at: '2026-10-21T00:00:00Z' }),
	memory('No emoji', 'preference', 1),
	memory('Siti is my sister in Bandung', 'person', 0.3, { needs_check: true }),
	memory('Gym Mon, Wed, Fri at 06:30', 'routine', 2),
];

describe('a memory’s strength', () => {
	it('is five steps of its weight: faint, weak, steady (where all start), strong, core', () => {
		expect([0.1, 0.6, 1, 2, 4.5].map((weight) => strength(weight))).toEqual([
			{ level: 1, name: 'faint' },
			{ level: 2, name: 'weak' },
			{ level: 3, name: 'steady' },
			{ level: 4, name: 'strong' },
			{ level: 5, name: 'core' },
		]);
	});
});

describe('the topics', () => {
	it('group the memories by kind, the strongest first, each with what it knows and when it last changed', () => {
		const topics = memoryTopics(memories, 'all', '');
		expect(topics.map((topic) => [topic.kind, topic.memories.map((one) => one.id), topic.updated])).toEqual([
			['preference', ['Short, direct answers', 'No emoji'], '2026-10-21T00:00:00Z'],
			['routine', ['Gym Mon, Wed, Fri at 06:30'], '2026-10-10T00:00:00Z'],
			['person', ['Siti is my sister in Bandung'], '2026-10-10T00:00:00Z'],
		]);
		expect(topics[0]).toMatchObject({ summary: 'Short, direct answers; No emoji', strength: { level: 4, name: 'strong' } });
		expect(topics[2]).toMatchObject({ fading: true });
	});

	it('narrow to core or fading memories, or a search', () => {
		expect(memoryTopics(memories, 'core', '').flatMap((topic) => topic.memories.map((one) => one.id))).toEqual(['Short, direct answers']);
		expect(memoryTopics(memories, 'fading', '').flatMap((topic) => topic.memories.map((one) => one.id))).toEqual(['Siti is my sister in Bandung']);
		expect(memoryTopics(memories, 'all', 'gym').flatMap((topic) => topic.memories.map((one) => one.id))).toEqual(['Gym Mon, Wed, Fri at 06:30']);
	});

	it('count each filter', () => {
		expect(memoryCounts(memories)).toEqual({ all: 4, core: 1, fading: 1, topics: 3 });
	});
});
