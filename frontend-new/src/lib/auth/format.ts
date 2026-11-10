// What the sign-in screens show of the API's answers: the hidden address, the resend countdown,
// which mail app to offer.

/** The address the API hid (`k******@gmail.com`), with the design's dots (`k••••••@gmail.com`). */
export function displayEmail(masked: string): string {
	return masked.replace(/\*/g, '•');
}

/** Seconds as the design writes a countdown: `0:42`, `1:05`. */
export function countdown(seconds: number): string {
	const whole = Math.max(0, Math.ceil(seconds));
	return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
}

/** Whole seconds from `now` until `at` (both in ms), never below zero. */
export function secondsUntil(at: number, now: number): number {
	return Math.max(0, Math.ceil((at - now) / 1000));
}

export type MailApp = 'gmail' | 'outlook';

const GMAIL = /^(gmail|googlemail)\.com$/;
const OUTLOOK = /^(outlook|hotmail|live|msn)\.(com|[a-z]{2}|co\.[a-z]{2}|com\.[a-z]{2})$/;

/** The webmail to offer a shortcut to, from the address's domain (shown or hidden), if any. */
export function mailApp(email: string): MailApp | null {
	const domain = email.split('@')[1]?.trim().toLowerCase() ?? '';
	if (GMAIL.test(domain)) return 'gmail';
	if (OUTLOOK.test(domain)) return 'outlook';
	return null;
}

/** An address the backend would take: something@domain.tld, no spaces (`valid_email` in orgs.rs). */
export function isEmail(value: string): boolean {
	const [local, domain, ...rest] = value.split('@');
	return (
		rest.length === 0 &&
		Boolean(local) &&
		domain !== undefined &&
		domain.includes('.') &&
		!domain.startsWith('.') &&
		!domain.endsWith('.') &&
		!/\s/.test(value)
	);
}
