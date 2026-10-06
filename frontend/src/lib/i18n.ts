import { isLocale, locales, type Locale } from '$lib/paraglide/runtime';

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
