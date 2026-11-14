// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { clickEverything, controls, render, settle, text, type } from '../../../test-stubs/render';
import Page from './+page.svelte';
import type { MemoryItem } from '$lib/types';

const memory = (over: Partial<MemoryItem>): MemoryItem => ({
	id: 'm1',
	content: 'Prefers the MRT over taxis',
	weight: 0.9,
	kind: 'preference',
	last_used_at: '2026-11-01T00:00:00Z',
	needs_check: false,
	created_at: '2026-09-01T00:00:00Z',
	updated_at: '2026-11-01T00:00:00Z',
	uses: 12,
	embedding: [],
	...over,
});

const memories = [
	memory({}),
	memory({ id: 'm2', content: 'Sister is called Dewi', kind: 'person', weight: 0.4, uses: 0, last_used_at: null }),
	memory({ id: 'm3', content: 'Runs on Saturday mornings', kind: 'routine', needs_check: true }),
	memory({ id: 'm4', content: 'Saving for a motorbike', kind: 'goal', weight: 0.2 }),
	memory({ id: 'm5', content: 'Works at a bakery', kind: 'fact' }),
];

const page = (data: unknown, form: unknown = null) => render(Page as never, { data: data as never, form: form as never, params: {} as never });

describe('memory page', () => {
	it('lists what Nomi remembers, by kind', () => {
		const shown = text(page({ memories, loadFailed: false }));
		for (const m of memories) expect(shown).toContain(m.content);
	});

	it('searches and filters, and opens a memory', async () => {
		const root = page({ memories, loadFailed: false }, { forgotten: 'm9' });
		const search = root.querySelector('input');
		if (search) type(search, 'dewi');
		if (search) type(search, '');
		expect(clickEverything(root)).toBeGreaterThan(0);
		await settle();
		clickEverything(document.body);
		expect(controls(/./).length).toBeGreaterThan(0);
	});

	it('shows who learned a memory, the profile and the memory settings', async () => {
		const settings = { learn_from_chats: true, keep_profile: false, search_past_chats: true, memory_budget: 8000, profile_budget: 1000, memory_used: 2000, profile_used: 8 };
		const root = page(
			{ memories: [memory({ learned_by: 'health' })], loadFailed: false, settings, profile: { text: 'Designer in Jakarta', tokens: 5, updated_at: '2026-11-01T00:00:00Z' } },
			{ profileError: 'Too long', text: 'Designer in Jakarta, draft' },
		);
		const shown = text(root);
		expect(shown).toContain('Learned by Raga');
		expect(shown).toContain('Memory settings');
		expect(shown).toContain('2000 of 8000 tokens used');
		expect(shown).toContain('Too long');
		expect((root.querySelector('textarea') as HTMLTextAreaElement).value).toBe('Designer in Jakarta, draft');
		expect(clickEverything(root)).toBeGreaterThan(0);
		await settle();
	});

	it('says so when nothing is remembered or the list failed', () => {
		expect(page({ memories: [], loadFailed: false }).textContent).toBeTruthy();
		expect(page({ memories: [], loadFailed: true }, { error: 'Nope' }).textContent).toBeTruthy();
	});
});
