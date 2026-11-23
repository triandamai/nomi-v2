import { describe, expect, it } from 'vitest';
import type { BoardColumn, ProjectIssue } from '$lib/adapters/api';
import { boardColumns, crewMember, dropIndex, moveOnBoard, overdue } from './issues';

const issue = (number: number, more: Partial<ProjectIssue> = {}): ProjectIssue => ({
	number,
	title: `Issue ${number}`,
	kind: 'bug',
	severity: 'high',
	status: 'open',
	column_id: 'backlog',
	position: number,
	due_on: null,
	points: null,
	crew_agent: 'coding',
	assigned_by_agent: null,
	crew_state: null,
	session_id: null,
	...more,
});
const board = (): BoardColumn[] => [
	{ id: 'backlog', name: 'backlog', position: 0, fixed: 'backlog', issues: [issue(1), issue(2, { due_on: '2026-01-01', points: 3 })] },
	{ id: 'ready', name: 'ready', position: 1, fixed: 'ready', issues: [issue(3, { crew_state: 'failed', session_id: 's 3' })] },
	{ id: 'qa', name: 'QA', position: 2, fixed: null, issues: [issue(4, { crew_state: 'working', assigned_by_agent: 'planning' })] },
];

describe('issue board', () => {
	it('names the crew by agent type', () => {
		expect(crewMember('coding')).toEqual({ crew: 'koda', name: 'Koda' });
		expect(crewMember('inspector')).toEqual({ crew: null, name: 'Iris' });
		expect(crewMember('scout')).toEqual({ crew: null, name: 'Scout' });
	});

	it('moves a card while the move is saved', () => {
		const moved = moveOnBoard(board(), issue(1), 'qa', 0);
		expect(moved[0].issues.map((one) => one.number)).toEqual([2]);
		expect(moved[2].issues.map((one) => one.number)).toEqual([1, 4]);
		expect(moved[2].issues[0].column_id).toBe('qa');
		expect(moveOnBoard(board(), issue(1), 'backlog', 99)[0].issues.map((one) => one.number)).toEqual([2, 1]);
	});

	it('lands a dropped card above the card under its middle', () => {
		const cards = [
			{ top: 0, height: 40 },
			{ top: 50, height: 40 },
		];
		expect(dropIndex(cards, 10)).toBe(0);
		expect(dropIndex(cards, 60)).toBe(1);
		expect(dropIndex(cards, 200)).toBe(2);
		expect(overdue('2026-01-01', '2026-02-01')).toBe(true);
		expect(overdue(null, '2026-02-01')).toBe(false);
	});

	it('says what each card is and how the crew is getting on', () => {
		const columns = boardColumns(board(), '2026-11-21', (day) => day);
		expect(columns.map((column) => column.name)).toEqual(['Backlog', 'Ready to dev', 'QA']);
		expect(columns[1].hint).toBe('The crew picks these up in order.');
		expect(columns[0].cards[1]).toMatchObject({ tags: 'Bug · High · 3 pts', status: { text: 'Overdue since 2026-01-01', tone: 'late' } });
		expect(columns[1].cards[0]).toMatchObject({ retry: true, chatHref: '/chat/s%203', status: { tone: 'failed', crew: 'koda' } });
		expect(columns[2].cards[0].status?.text).toBe('Koda is working on it (from Rena)');
		expect(columns[0].cards[0].status).toBeUndefined();
	});
});

describe('localDay', () => {
	it('is the date in the given zone', async () => {
		const { localDay } = await import('./issues');
		expect(localDay(new Date('2026-11-21T20:00:00Z'), 'Asia/Jakarta')).toBe('2026-11-22');
		expect(localDay(new Date('2026-11-21T20:00:00Z'), 'UTC')).toBe('2026-11-21');
	});
});
