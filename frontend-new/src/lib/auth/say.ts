import { m } from '$lib/paraglide/messages';
import type { Alert, AlertKind, FieldProblem, GoogleError } from './problems';

// The sign-in screens' problems in the person's language: what happened, and what to do next.

const FIELDS: Record<FieldProblem, () => string> = {
	email_required: m.err_email_required,
	email_invalid: m.err_email_invalid,
	password_required: m.err_password_required,
	password_short: m.err_password_short,
	password_mismatch: m.err_password_mismatch,
	wrong_credentials: m.signin_wrong,
	email_taken: m.register_email_taken,
	workspace_required: m.register_workspace_required,
	invite_invalid: m.register_invite_invalid,
	name_required: m.firstrun_name_required,
};

export function sayField(problem: FieldProblem | undefined): string | undefined {
	return problem && FIELDS[problem]();
}

const ALERTS: Record<AlertKind, () => string> = {
	too_many_codes: m.signin_too_many_codes,
	email_failed: m.signin_email_failed,
	register_email_failed: m.register_email_failed,
	sign_in_failed: m.signin_failed,
	register_failed: m.register_failed,
	forgot_failed: m.forgot_failed,
	reset_failed: m.reset_failed,
	firstrun_failed: m.firstrun_failed,
	unreachable: m.auth_unreachable,
};

export function sayAlert(alert: Alert): string {
	return ALERTS[alert.kind]();
}

/** The API's own reason, as the error callout quotes it: `502 · email_failed`. */
export function alertCode(alert: Alert): string | undefined {
	return [alert.status, alert.code].filter(Boolean).join(' · ') || undefined;
}

const GOOGLE: Record<GoogleError, () => string> = {
	cancelled: m.google_err_cancelled,
	unavailable: m.google_err_unavailable,
	conflict: m.google_err_conflict,
	expired: m.google_err_expired,
	unverified: m.google_err_unverified,
	invite: m.google_err_invite,
	failed: m.google_err_failed,
};

export function sayGoogle(error: GoogleError): string {
	return GOOGLE[error]();
}
