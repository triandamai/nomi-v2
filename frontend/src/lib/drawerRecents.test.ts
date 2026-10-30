import { describe, expect, it, vi } from 'vitest';
import {
	DEFAULT_RECENTS_LIMIT,
	isRecentChatActive,
	loadRecentChats,
	parseRecentsForm,
	refreshesAfterNavigation,
	resolveRecents,
	toRecentChats,
} from './drawerRecents';
import type { SessionSummary } from './types';

function session(id: string, title: string | null, projectId: string | null = null, pinnedAt: string | null = null): SessionSummary {
	return {
		id,
		channel: 'web',
		chat_type: 'dm',
		chat_id: id,
		title,
		last_message: null,
		agent_active: false,
		updated_at: '2026-10-18T00:00:00Z',
		project_id: projectId,
		pinned: pinnedAt !== null,
		pinned_at: pinnedAt,
		in_project_id: null,
	};
}

describe('resolveRecents', () => {
	it('shows ten recent chats until the person changes it', () => {
		expect(resolveRecents({})).toEqual({ enabled: true, limit: DEFAULT_RECENTS_LIMIT });
		expect(DEFAULT_RECENTS_LIMIT).toBe(10);
	});

	it('keeps a saved choice and ignores a size it does not offer', () => {
		expect(resolveRecents({ drawer_recents: false, drawer_recents_limit: 5 })).toEqual({ enabled: false, limit: 5 });
		expect(resolveRecents({ drawer_recents: true, drawer_recents_limit: 7 })).toEqual({ enabled: true, limit: 10 });
	});
});

describe('toRecentChats', () => {
	it('lists chats newest first up to the limit, leaving project sessions to Projects', () => {
		const sessions = [session('a', 'Trip plan'), session('p', 'Site', 'project-1'), session('b', 'Budget'), session('c', 'Gym')];
		expect(toRecentChats(sessions, 2)).toEqual([
			{ id: 'a', title: 'Trip plan', href: '/chat/a', pinned: false },
			{ id: 'b', title: 'Budget', href: '/chat/b', pinned: false },
		]);
	});

	it('keeps pinned chats on top, newest pin first, and always shows them', () => {
		const sessions = [
			session('a', 'Trip plan'),
			session('b', 'Budget', null, '2026-10-01T00:00:00Z'),
			session('c', 'Gym'),
			session('d', 'Taxes', null, '2026-10-20T00:00:00Z'),
		];
		expect(toRecentChats(sessions, 1).map((chat) => [chat.id, chat.pinned])).toEqual([
			['d', true],
			['b', true],
			['a', false],
		]);
	});

	it('names an untitled chat the way the Chats page does', () => {
		expect(toRecentChats([session('a', null)], 10)[0].title).toBe('New chat');
	});
});

describe('isRecentChatActive', () => {
	it('marks only the open chat', () => {
		expect(isRecentChatActive('/chat/a', 'a')).toBe(true);
		expect(isRecentChatActive('/chat/ab', 'a')).toBe(false);
		expect(isRecentChatActive('/chats', 'a')).toBe(false);
	});
});

describe('refreshesAfterNavigation', () => {
	const at = (pathname: string) => ({ url: new URL(`https://nomi.test${pathname}`) });

	it('refreshes when moving to another page, where a new or renamed chat may show up', () => {
		expect(refreshesAfterNavigation(at('/chat/new'), at('/chat/a'))).toBe(true);
	});

	it('leaves the first load to mounting, which also covers a drawer opened later', () => {
		expect(refreshesAfterNavigation(null, at('/chat/a'))).toBe(false);
	});

	it('ignores staying on the same page (a search or hash change)', () => {
		expect(refreshesAfterNavigation(at('/chats'), at('/chats'))).toBe(false);
	});
});

describe('loadRecentChats', () => {
	it('reads the recent sessions and keeps the chats', async () => {
		const fetchFn = vi.fn(async () => Response.json({ sessions: [session('a', 'One'), session('b', 'Two')] }));
		expect(await loadRecentChats(fetchFn, 1)).toEqual({ status: 'ready', chats: [{ id: 'a', title: 'One', href: '/chat/a', pinned: false }] });
		expect(fetchFn).toHaveBeenCalledWith('/chats/recent');
	});

	it('reports an error instead of an empty list when the sessions cannot be read', async () => {
		expect(await loadRecentChats(async () => new Response('nope', { status: 500 }), 10)).toEqual({ status: 'error' });
		expect(
			await loadRecentChats(async () => {
				throw new TypeError('offline');
			}, 10),
		).toEqual({ status: 'error' });
	});
});

describe('parseRecentsForm', () => {
	function form(fields: Record<string, string>): FormData {
		const data = new FormData();
		for (const [key, value] of Object.entries(fields)) data.set(key, value);
		return data;
	}

	it('reads the switch and the size', () => {
		expect(parseRecentsForm(form({ drawer_recents: 'true', drawer_recents_limit: '15' }))).toEqual({ drawer_recents: true, drawer_recents_limit: 15 });
		expect(parseRecentsForm(form({ drawer_recents: 'false', drawer_recents_limit: '5' }))).toEqual({ drawer_recents: false, drawer_recents_limit: 5 });
	});

	it('rejects a size it does not offer', () => {
		expect(parseRecentsForm(form({ drawer_recents: 'true', drawer_recents_limit: '12' }))).toBeNull();
		expect(parseRecentsForm(form({ drawer_recents: 'true' }))).toBeNull();
	});
});
