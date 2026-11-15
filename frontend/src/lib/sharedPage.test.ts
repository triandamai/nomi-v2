import { describe, expect, it } from 'vitest';
import { isReportReason, reasonLabel, REPORT_REASONS, shareSummary, sharedMeta, viewsLabel } from './sharedPage';

describe('shared page', () => {
	it('says who shared it and when', () => {
		expect(sharedMeta('M. P.', '2026-10-22T09:00:00Z', 'en')).toBe('Shared by M. P. · Oct 22, 2026');
		expect(sharedMeta('', '2026-10-22T09:00:00Z', 'en')).toBe('Oct 22, 2026');
	});

	it('counts the owner’s views', () => {
		expect([viewsLabel(0), viewsLabel(1), viewsLabel(3)]).toEqual(['No views yet.', '1 view so far.', '3 views so far.']);
	});

	it('knows the reasons a page can be reported for', () => {
		expect(REPORT_REASONS.every(isReportReason)).toBe(true);
		expect([isReportReason('boring'), isReportReason(null)]).toEqual([false, false]);
		expect(reasonLabel('personal_info')).toBe("Someone's private information");
	});

	it('summarises a listed link', () => {
		expect(shareSummary({ scope: 'chat', anyone: true, mask: true, expires_at: null }, 'en')).toBe('Whole chat · Anyone with the link · Amounts hidden · Never expires');
		expect(shareSummary({ scope: 'reply', anyone: false, mask: false, expires_at: '2026-10-22T09:00:00Z' }, 'en')).toBe(
			'One reply · Signed-in people · Expires Oct 22, 2026',
		);
	});
});
