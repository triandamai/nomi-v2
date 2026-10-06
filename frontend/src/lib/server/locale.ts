import type { Cookies } from '@sveltejs/kit';
import { cookieMaxAge, cookieName, getLocale } from '$lib/paraglide/runtime';
import { isLanguage, type Locale } from '$lib/i18n';
import { apiFetch } from '$lib/server/api';

/** Remembers the person's language for the next requests (Paraglide reads this cookie). */
export function setLanguageCookie(cookies: Cookies, language: Locale): void {
	cookies.set(cookieName, language, { path: '/', maxAge: cookieMaxAge, httpOnly: false, sameSite: 'lax' });
}

/**
 * Right after signing in: an account that already has a language keeps it on this device too;
 * a brand-new one starts in the language the sign-up pages were shown in.
 */
export async function settleLanguageAfterSignIn(fetch: typeof globalThis.fetch, cookies: Cookies, isNew: boolean): Promise<void> {
	try {
		if (isNew) {
			await apiFetch(fetch, cookies, '/api/preferences', { method: 'PUT', body: JSON.stringify({ language: getLocale() }) });
			return;
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences');
		if (!response.ok) return;
		const { language } = (await response.json()) as { language?: string };
		if (isLanguage(language)) setLanguageCookie(cookies, language);
	} catch {
		// Best-effort: the app's layout catches up on the next page.
	}
}
