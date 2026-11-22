import type { BoardColumn, ProjectIssue } from '$lib/adapters/api';
import type { CrewId } from '$lib/ds/agents/crew';
import type { IssueCard, IssueColumn } from '$lib/ds/projects/IssueBoard.svelte';
import { m } from '$lib/paraglide/messages';

// The Issues sheet (the old app's issue board, rebuilt): who works an issue, where a moved card
// lands, and what a card says about how it's going.

/** Crew members by their agent type, as the backend names them. */
const CREW: Record<string, { crew: CrewId; name: string }> = {
	coding: { crew: 'koda', name: 'Koda' },
	planning: { crew: 'rena', name: 'Rena' },
	money: { crew: 'dana', name: 'Dana' },
	reminders: { crew: 'kala', name: 'Kala' },
	workspace: { crew: 'tara', name: 'Tara' },
	files: { crew: 'maya', name: 'Maya' },
	health: { crew: 'raga', name: 'Raga' },
	chitchat: { crew: 'nomi', name: 'Nomi' },
};

/** A crew member by agent type; one off the list goes by their type, capitalised. */
export function crewMember(agent: string): { crew: CrewId | null; name: string } {
	if (CREW[agent]) return CREW[agent];
	if (agent === 'inspector') return { crew: null, name: 'Iris' };
	return { crew: null, name: agent.charAt(0).toUpperCase() + agent.slice(1) };
}

/** The board with `issue` moved to `columnId` at `index`, as the server will have it: shown while the move is saved. */
export function moveOnBoard(columns: BoardColumn[], issue: ProjectIssue, columnId: string, index: number): BoardColumn[] {
	return columns.map((column) => {
		const issues = column.issues.filter((one) => one.number !== issue.number);
		if (column.id === columnId) issues.splice(Math.min(Math.max(index, 0), issues.length), 0, { ...issue, column_id: columnId });
		return { ...column, issues };
	});
}

/** Where a card dropped at `y` lands among a column's other cards (their top and height, in order). */
export function dropIndex(cards: { top: number; height: number }[], y: number): number {
	const at = cards.findIndex((card) => y < card.top + card.height / 2);
	return at === -1 ? cards.length : at;
}

/** Whether a deadline (YYYY-MM-DD) has passed by `today` (YYYY-MM-DD). */
export function overdue(due: string | null, today: string): boolean {
	return due !== null && due < today;
}

const KIND: Record<ProjectIssue['kind'], () => string> = { task: m.issues_kind_task, enhancement: m.issues_kind_enhancement, bug: m.issues_kind_bug };
const SEVERITY: Record<ProjectIssue['severity'], () => string> = { low: m.issues_severity_low, medium: m.issues_severity_medium, high: m.issues_severity_high, critical: m.issues_severity_critical };

/** A fixed column's own name and the line under it; the project's columns keep theirs. */
function columnHead(column: BoardColumn): { name: string; hint?: string } {
	if (column.fixed === 'backlog') return { name: m.board_backlog() };
	if (column.fixed === 'ready') return { name: m.board_ready(), hint: m.board_ready_hint() };
	if (column.fixed === 'done') return { name: m.board_done_column(), hint: m.board_done_hint() };
	if (column.fixed === 'closed') return { name: m.board_closed_column() };
	return { name: column.name };
}

/** The card's quiet line: how the crew is getting on, else its deadline. */
function cardStatus(issue: ProjectIssue, column: BoardColumn, today: string, date: (day: string) => string): IssueCard['status'] {
	const who = crewMember(issue.crew_agent);
	const crew = who.crew ?? undefined;
	if (column.fixed === 'done') return issue.crew_state ? { text: m.board_done_by({ name: who.name }), crew } : undefined;
	if (column.fixed === 'closed') return undefined;
	if (issue.crew_state === 'queued') return { text: m.board_queued({ name: who.name }), crew };
	if (issue.crew_state === 'working')
		return { text: issue.assigned_by_agent ? m.board_working_from({ name: who.name, from: crewMember(issue.assigned_by_agent).name }) : m.board_working({ name: who.name }), crew };
	if (issue.crew_state === 'done') return { text: m.board_done({ name: who.name }), crew };
	if (issue.crew_state === 'failed') return { text: m.board_failed({ name: who.name }), tone: 'failed', crew };
	if (issue.due_on) return overdue(issue.due_on, today) ? { text: m.board_overdue({ date: date(issue.due_on) }), tone: 'late' } : { text: m.board_due({ date: date(issue.due_on) }) };
	return undefined;
}

/** The board as the Issues sheet's columns and cards. */
export function boardColumns(columns: BoardColumn[], today: string, date: (day: string) => string): IssueColumn[] {
	return columns.map((column) => ({
		id: column.id,
		...columnHead(column),
		cards: column.issues.map((issue) => ({
			number: issue.number,
			title: issue.title,
			tags: [KIND[issue.kind](), SEVERITY[issue.severity](), ...(issue.points ? [m.board_points({ count: issue.points })] : [])].join(' · '),
			status: cardStatus(issue, column, today, date),
			...(issue.session_id && { chatHref: `/chat/${encodeURIComponent(issue.session_id)}` }),
			...(issue.crew_state === 'failed' && column.fixed === 'ready' && { retry: true }),
		})),
	}));
}

/** The day (YYYY-MM-DD) it is now in the person's time zone, to say which deadlines have passed. */
export function localDay(now: Date, timeZone?: string): string {
	return new Intl.DateTimeFormat('en-CA', { timeZone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(now);
}
