import { describe, expect, it } from 'vitest';
import { searchGroups } from './search';

const now = new Date('2026-11-15T10:00:00Z');
const hit = (over: Partial<Parameters<typeof searchGroups>[0][number]>) => ({ kind: 'chat' as const, id: 'x', title: 'x', snippet: null, updated_at: '2026-11-15T09:00:00Z', ...over });

describe('what search found', () => {
	it('groups chats, memory and files, in that order, each opening where it lives', () => {
		const groups = searchGroups(
			[
				hit({ kind: 'file', id: 'src/routes/menu/+page.svelte', title: 'src/routes/menu/+page.svelte', project_id: 'p1', snippet: 'warung-kopi-menu' }),
				hit({ kind: 'chat', id: 's1', title: 'Budget belanja Oktober', snippet: '…beras naik 10%…', session_id: 's1' }),
				hit({ kind: 'memory', id: 'm1', title: 'Prefers kopi susu', snippet: null }),
			],
			now,
		);
		expect(groups.map((group) => group.kind)).toEqual(['chat', 'memory', 'file']);
		expect(groups[0].rows[0]).toMatchObject({ title: 'Budget belanja Oktober', snippet: '…beras naik 10%…', href: '/chat/s1' });
		expect(groups[1].rows[0]).toMatchObject({ href: '/settings/memory' });
		expect(groups[2].rows[0]).toMatchObject({ href: '/projects/p1?open=code&file=src%2Froutes%2Fmenu%2F%2Bpage.svelte' });
		expect(groups[0].rows[0].meta).toMatch(/ago|1h/);
	});

	it('leaves out empty groups and a file without its project', () => {
		expect(searchGroups([hit({ kind: 'file', id: 'a.ts', title: 'a.ts' })], now)).toEqual([]);
		expect(searchGroups([], now)).toEqual([]);
	});
});
