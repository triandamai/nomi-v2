import { m } from '$lib/paraglide/messages';
import { getLocale, isLocale, locales, type Locale } from '$lib/paraglide/runtime';

/** The languages Nomi speaks, each named in itself so anyone can find their own. */
export const LANGUAGES: { value: Locale; label: string; hint: string }[] = [
	{ value: 'en', label: 'English', hint: 'English' },
	{ value: 'id', label: 'Bahasa Indonesia', hint: 'Indonesian' },
];

export function isLanguage(value: unknown): value is Locale {
	return typeof value === 'string' && isLocale(value);
}

export { locales };
export type { Locale };

/** "just now", "45m ago", "yesterday", "4mo ago", in the person's language. */
export function timeAgo(iso: string, now: number = Date.now()): string {
	const minutes = Math.floor((now - new Date(iso).getTime()) / 60000);
	if (minutes < 1) return m.time_just_now();
	const format = new Intl.RelativeTimeFormat(getLocale(), { style: 'narrow', numeric: 'auto' });
	if (minutes < 60) return format.format(-minutes, 'minute');
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return format.format(-hours, 'hour');
	const days = Math.floor(hours / 24);
	if (days < 30) return format.format(-days, 'day');
	const months = Math.floor(days / 30);
	if (months < 12) return format.format(-months, 'month');
	return format.format(-Math.floor(months / 12), 'year');
}

/** The language to hear dictation in: the person's chosen language, in the browser's own variant when it matches. */
export function speechLanguage(): string {
	const browser = typeof navigator === 'undefined' ? '' : navigator.language;
	const locale = getLocale();
	if (browser.toLowerCase().startsWith(locale)) return browser;
	return locale === 'id' ? 'id-ID' : 'en-US';
}
