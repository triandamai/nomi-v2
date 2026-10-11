import { describe, expect, it } from 'vitest';
import { issuesHref, issuesQuery, readIssueFilters } from './issues';

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
