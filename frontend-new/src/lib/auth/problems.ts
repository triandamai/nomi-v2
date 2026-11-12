// What can go wrong on the sign-in screens, as codes: the server says which, the page says it in
// the person's language (say.ts). Codes, not sentences, so the tests read the same in any language.

/** A field's problem, shown as that field's error. */
export type FieldProblem =
	| 'email_required'
	| 'email_invalid'
	| 'password_required'
	| 'password_short'
	| 'password_mismatch'
	| 'wrong_credentials'
	| 'email_taken'
	| 'workspace_required'
	| 'invite_invalid'
	| 'name_required';

export type Fields = Partial<Record<'email' | 'password' | 'confirm' | 'workspace' | 'invite' | 'name', FieldProblem>>;

/** A problem with the whole form, shown above it: what happened, with the API's status and code. */
export type AlertKind =
	| 'too_many_codes'
	| 'email_failed'
	| 'register_email_failed'
	| 'sign_in_failed'
	| 'register_failed'
	| 'forgot_failed'
	| 'reset_failed'
	| 'firstrun_failed'
	| 'unreachable';

export type Alert = { kind: AlertKind; status?: number; code?: string };

/** Where the emailed code stands after an attempt. */
export type CodeProblem = 'incomplete' | 'wrong_code' | 'locked' | 'expired' | 'wait' | 'too_many_codes' | 'email_failed' | 'failed' | 'unreachable';

/** Why a Google sign-in came back without signing in (`?google_error=` on the sign-in page). */
export const GOOGLE_ERRORS = ['cancelled', 'unavailable', 'conflict', 'expired', 'unverified', 'invite', 'failed'] as const;
export type GoogleError = (typeof GOOGLE_ERRORS)[number];

/** The Google error a sign-in page was sent back with: one it knows, or "failed" for any other. */
export function googleErrorFrom(value: string | null | undefined): GoogleError | null {
	if (!value) return null;
	return (GOOGLE_ERRORS as readonly string[]).includes(value) ? (value as GoogleError) : 'failed';
}

/** Why an invite couldn't be accepted. */
export type InviteProblem = 'expired' | 'used' | 'other_address' | 'failed';

/** The shortest password the API takes on a reset; sign-up asks the same. */
export const MIN_PASSWORD_LENGTH = 8;
