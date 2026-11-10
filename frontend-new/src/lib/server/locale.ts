import type { Cookies } from '@sveltejs/kit';
import { cookieMaxAge, cookieName, getLocale, isLocale } from '$lib/paraglide/runtime';
import { apiFetch, type ApiEvent } from './api';

/** Remembers the person's language for the next requests (Paraglide reads this cookie). */
export function setLanguageCookie(cookies: Cookies, language: string): void {
	cookies.set(cookieName, language, { path: '/', maxAge: cookieMaxAge, httpOnly: false, sameSite: 'lax' });
}

/**
 * Right after signing in: an account that already has a language keeps it on this device too;
 * a brand-new one starts in the language the sign-up pages were shown in.
 */
export async function settleLanguageAfterSignIn(event: ApiEvent, isNew: boolean): Promise<void> {
	try {
		if (isNew) {
			await apiFetch(event, '/api/preferences', { method: 'PUT', body: JSON.stringify({ language: getLocale() }) });
			return;
		}
		const response = await apiFetch(event, '/api/preferences');
		if (!response.ok) return;
		const { language } = (await response.json()) as { language?: string };
		if (typeof language === 'string' && isLocale(language)) setLanguageCookie(event.cookies, language);
	} catch {
		// Best-effort: the app's layout catches up on the next page.
	}
}
