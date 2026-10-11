// Filters for a project's issues page, read from and written to its URL.

import type { IssueKind, IssueSeverity } from '$lib/types';

export const ISSUE_KINDS: IssueKind[] = ['feature', 'enhancement', 'bug'];
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
