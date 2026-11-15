// What a shared page (routes/shared/[token]) says about its link: who shared it and when, the
// owner's view count, and why a viewer may report it.
import { m } from '$lib/paraglide/messages';

/** Why a viewer flags a shared page (backend: routes/shares.rs `REPORT_REASONS`). */
export const REPORT_REASONS = ['spam', 'abuse', 'personal_info', 'illegal', 'other'] as const;
export type ReportReason = (typeof REPORT_REASONS)[number];

export function isReportReason(value: unknown): value is ReportReason {
	return typeof value === 'string' && (REPORT_REASONS as readonly string[]).includes(value);
}

/** A reason as the report form lists it. */
export function reasonLabel(reason: ReportReason): string {
	return {
		spam: m.shared_reason_spam,
		abuse: m.shared_reason_abuse,
		personal_info: m.shared_reason_personal_info,
		illegal: m.shared_reason_illegal,
		other: m.shared_reason_other,
	}[reason]();
}

/** The meta line: "Shared by M. · 22 Oct 2026". */
export function sharedMeta(by: string, at: string, locale: string): string {
	const when = new Date(at).toLocaleDateString(locale, { day: 'numeric', month: 'short', year: 'numeric' });
	return by ? m.shared_by({ who: by, when }) : when;
}

/** The owner's banner: "3 views so far". */
export function viewsLabel(views: number): string {
	if (views === 0) return m.shared_views_none();
	return views === 1 ? m.shared_views_one() : m.shared_views({ count: views });
}

/** A share link as Settings › Data lists it (backend: `GET /api/shares`). */
export interface ShareLinkSummary {
	scope: 'reply' | 'chat';
	mask: boolean;
	anyone: boolean;
	expires_at: string | null;
}

/** What a listed link shares, with whom, and until when: "Whole chat · Anyone with the link · Amounts hidden · Expires 22 Oct 2026". */
export function shareSummary(share: ShareLinkSummary, locale: string): string {
	const until = share.expires_at
		? m.account_share_expires({ when: new Date(share.expires_at).toLocaleDateString(locale, { day: 'numeric', month: 'short', year: 'numeric' }) })
		: m.account_share_never();
	return [
		share.scope === 'reply' ? m.account_share_reply() : m.account_share_chat(),
		share.anyone ? m.account_share_anyone() : m.account_share_signed_in(),
		share.mask ? m.account_share_masked() : null,
		until,
	]
		.filter(Boolean)
		.join(' · ');
}
