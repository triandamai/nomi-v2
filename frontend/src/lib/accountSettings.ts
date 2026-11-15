/**
 * Small rules behind Settings: the notification preferences a form sends, quiet hours, how the
 * chat composer reads Enter, and the number and date formats a person can pick.
 */
import type { QuietHours } from './types';

/** Where quiet hours start when they're first turned on (the pen's "22:00–06:00"). */
export const DEFAULT_QUIET_HOURS: QuietHours = { start: '22:00', end: '06:00' };

const TIME = /^([01]\d|2[0-3]):[0-5]\d$/;

/** Whether `value` is a "HH:MM" time. */
export function isTime(value: string): boolean {
	return TIME.test(value);
}

/**
 * The body of `PUT /api/notifications/preferences` from the Inbox's settings form: the two email
 * switches, reminder pushes, and quiet hours (`quiet` on, with `quiet_start`/`quiet_end`), or null.
 */
export function notificationPreferencesBody(form: FormData): Record<string, unknown> {
	const on = (name: string) => form.get(name) === 'true';
	const start = String(form.get('quiet_start') ?? '');
	const end = String(form.get('quiet_end') ?? '');
	const quiet = on('quiet') && isTime(start) && isTime(end) && start !== end ? { start, end } : null;
	return { email_account: on('email_account'), email_promos: on('email_promos'), push_reminders: on('push_reminders'), quiet_hours: quiet };
}

/**
 * Whether a key press sends the message. With `enterSends` (the default) Enter sends and
 * Shift+Enter makes a new line; without it Enter makes a new line and ⌘/Ctrl+Enter sends.
 */
export function sendsOnEnter(event: Pick<KeyboardEvent, 'key' | 'shiftKey' | 'metaKey' | 'ctrlKey' | 'isComposing'>, enterSends: boolean): boolean {
	if (event.key !== 'Enter' || event.isComposing || event.shiftKey) return false;
	return enterSends || event.metaKey || event.ctrlKey;
}

/** The regions numbers and dates can follow; '' follows the app's language. */
export const FORMAT_REGIONS = ['', 'en-US', 'en-GB', 'en-AU', 'id-ID', 'de-DE', 'fr-FR', 'ja-JP'] as const;

/** A sample of how a region writes a number and a date, e.g. "1,234.5 · 14/11/2026". */
export function formatSample(region: string, fallback: string, at: Date = new Date(2026, 10, 14)): string {
	const locale = region || fallback;
	const number = new Intl.NumberFormat(locale).format(1234.5);
	const date = new Intl.DateTimeFormat(locale, { dateStyle: 'short' }).format(at);
	return `${number} · ${date}`;
}
