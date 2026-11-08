// Filters for a project's issues page, read from and written to its URL.

import type { CrewId } from '$lib/components/ds/agents/crew';
import type { BoardColumn, IssueKind, IssueSeverity, ProjectIssue } from '$lib/types';

export const ISSUE_KINDS: IssueKind[] = ['task', 'enhancement', 'bug'];
export const ISSUE_SEVERITIES: IssueSeverity[] = ['critical', 'high', 'medium', 'low'];

export interface IssueFilters {
	status: 'open' | 'closed';
	kind: IssueKind | null;
	severity: IssueSeverity | null;
}

export function readIssueFilters(params: URLSearchParams): IssueFilters {
	const kind = params.get('kind');
	const severity = params.get('severity');
	return {
		status: params.get('status') === 'closed' ? 'closed' : 'open',
		kind: ISSUE_KINDS.includes(kind as IssueKind) ? (kind as IssueKind) : null,
		severity: ISSUE_SEVERITIES.includes(severity as IssueSeverity) ? (severity as IssueSeverity) : null,
	};
}

/** The query string for `filters`, leaving out the defaults. */
export function issuesQuery(filters: IssueFilters): string {
	const params = new URLSearchParams();
	if (filters.status !== 'open') params.set('status', filters.status);
	if (filters.kind) params.set('kind', filters.kind);
	if (filters.severity) params.set('severity', filters.severity);
	return params.toString();
}

/** The page's own link for `filters` with `change` applied. */
export function issuesHref(filters: IssueFilters, change: Partial<IssueFilters>): string {
	const query = issuesQuery({ ...filters, ...change });
	return query ? `?${query}` : '?';
}

/** Estimates the points field suggests. */
export const POINT_SUGGESTIONS = [1, 2, 3, 5, 8, 13];
/** The most points one issue can be estimated at (the backend refuses more). */
export const MAX_POINTS = 100;
/** Who works an issue unless someone picks another. */
export const DEFAULT_CREW_AGENT = 'coding';

/** The crew members an issue can go to: their agent type and who they are in the design. */
export const ISSUE_CREW: { agent: string; crew: CrewId; name: string }[] = [
	{ agent: 'coding', crew: 'koda', name: 'Koda' },
	{ agent: 'planning', crew: 'rena', name: 'Rena' },
	{ agent: 'money', crew: 'dana', name: 'Dana' },
	{ agent: 'reminders', crew: 'kala', name: 'Kala' },
	{ agent: 'workspace', crew: 'tara', name: 'Tara' },
	{ agent: 'files', crew: 'maya', name: 'Maya' },
	{ agent: 'health', crew: 'raga', name: 'Raga' },
	{ agent: 'chitchat', crew: 'nomi', name: 'Nomi' },
];

/** A crew member by agent type; one outside the list goes by their type, capitalised. */
export function crewMember(agent: string): { crew: CrewId | null; name: string } {
	const found = ISSUE_CREW.find((member) => member.agent === agent);
	if (found) return { crew: found.crew, name: found.name };
	if (agent === 'inspector') return { crew: null, name: 'Iris' };
	return { crew: null, name: agent.charAt(0).toUpperCase() + agent.slice(1) };
}

/** Whether a deadline (YYYY-MM-DD) has passed by `today` (YYYY-MM-DD). */
export function overdue(due: string | null, today: string): boolean {
	return due !== null && due < today;
}

/** Today as YYYY-MM-DD in the person's own time zone. */
export function localToday(now = new Date()): string {
	const offset = now.getTimezoneOffset() * 60_000;
	return new Date(now.getTime() - offset).toISOString().slice(0, 10);
}

/** Whether a start date and deadline (YYYY-MM-DD, either empty) are in order. */
export function datesInOrder(start: string, due: string): boolean {
	return !start || !due || start <= due;
}

/**
 * The board with `issue` moved to `columnId` at `index`, as the server will have it: shown
 * straight away while the move is saved.
 */
export function moveOnBoard(columns: BoardColumn[], issue: ProjectIssue, columnId: string, index: number): BoardColumn[] {
	const without = columns.map((column) => ({ ...column, issues: column.issues.filter((i) => i.number !== issue.number) }));
	return without.map((column) => {
		if (column.id !== columnId) return column;
		const issues = [...column.issues];
		issues.splice(Math.min(Math.max(index, 0), issues.length), 0, { ...issue, column_id: columnId });
		return { ...column, issues };
	});
}

/** Where a card dropped at `y` lands among a column's cards (their top and height, in order). */
export function dropIndex(cards: { top: number; height: number }[], y: number): number {
	const at = cards.findIndex((card) => y < card.top + card.height / 2);
	return at === -1 ? cards.length : at;
}

/** The crew members whose name or role (`role`) has a word starting with `query`. */
export function crewMatches(query: string, role: (agent: string) => string): typeof ISSUE_CREW {
	const q = query.trim().toLowerCase();
	if (!q) return ISSUE_CREW;
	return ISSUE_CREW.filter((member) =>
		[member.name, role(member.agent)].some((text) =>
			text
				.toLowerCase()
				.split(/[\s,/]+/)
				.some((word) => word.startsWith(q)),
		),
	);
}

/** The longest description an issue can have (the backend refuses more). */
export const MAX_BODY_CHARS = 20_000;

/** The most crew-work rows the project chat shows; the rest are on the board. */
export const MAX_WORK_ROWS = 3;

/**
 * The issues the crew has taken from Ready to dev (waiting, working, done or stuck), in the
 * board's order, for the project chat to show.
 */
export function crewWork(columns: BoardColumn[]): ProjectIssue[] {
	return columns.flatMap((column) => column.issues).filter((issue) => issue.crew_state !== null);
}
