import { describe, expect, it } from 'vitest';
import { crewMember, datesInOrder, dropIndex, issuesHref, issuesQuery, localToday, moveOnBoard, overdue, readIssueFilters } from './issues';
import type { BoardColumn, ProjectIssue } from './types';

describe('issue filters', () => {
	it('defaults to open issues of every type and severity', () => {
		expect(readIssueFilters(new URLSearchParams())).toEqual({ status: 'open', kind: null, severity: null });
	});

	it('ignores values it does not know', () => {
		expect(readIssueFilters(new URLSearchParams('status=done&kind=chore&severity=urgent'))).toEqual({
			status: 'open',
			kind: null,
			severity: null,
		});
	});

	it('leaves the defaults out of the link', () => {
		const filters = readIssueFilters(new URLSearchParams('kind=bug'));
		expect(issuesQuery(filters)).toBe('kind=bug');
		expect(issuesHref(filters, { status: 'closed' })).toBe('?status=closed&kind=bug');
		expect(issuesHref(filters, { kind: null })).toBe('?');
	});
});

describe('issue board', () => {
	const issue = (number: number, column: string) => ({ number, column_id: column }) as ProjectIssue;
	const columns: BoardColumn[] = [
		{ id: 'a', name: 'Backlog', position: 0, fixed: 'backlog', issues: [issue(1, 'a'), issue(2, 'a')] },
		{ id: 'b', name: 'Ready to dev', position: 1, fixed: 'ready', issues: [issue(3, 'b')] },
	];

	it('moves an issue to its place in another column', () => {
		const moved = moveOnBoard(columns, columns[0].issues[0], 'b', 0);
		expect(moved.map((c) => c.issues.map((i) => i.number))).toEqual([[2], [1, 3]]);
		expect(moved[1].issues[0].column_id).toBe('b');
		expect(moveOnBoard(columns, columns[0].issues[0], 'a', 9).map((c) => c.issues.map((i) => i.number))).toEqual([[2, 1], [3]]);
	});

	it('drops a card above the first card whose middle is below the pointer', () => {
		const cards = [{ top: 0, height: 40 }, { top: 50, height: 40 }];
		expect(dropIndex(cards, 10)).toBe(0);
		expect(dropIndex(cards, 30)).toBe(1);
		expect(dropIndex(cards, 200)).toBe(2);
	});

	it('names the crew, and knows late deadlines and dates out of order', () => {
		expect(crewMember('money')).toEqual({ crew: 'dana', name: 'Dana' });
		expect(crewMember('inspector').name).toBe('Iris');
		expect(overdue('2026-11-07', '2026-11-08')).toBe(true);
		expect(overdue('2026-11-08', '2026-11-08')).toBe(false);
		expect(overdue(null, '2026-11-08')).toBe(false);
		expect(datesInOrder('2026-11-09', '2026-11-12')).toBe(true);
		expect(datesInOrder('2026-11-12', '2026-11-09')).toBe(false);
		expect(datesInOrder('', '2026-11-09')).toBe(true);
		expect(localToday(new Date(2026, 10, 8, 23, 30))).toBe('2026-11-08');
	});
});
