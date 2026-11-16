// A phone, for the screens Flow / Mobile Chat and Mobile Build draw on their own (below 768): the
// server's first guess from the browser's name, so a phone gets its own screen on the first paint;
// the page then follows the window's real width (lib/phone.svelte.ts).

/** Whether the browser says it's a phone (a tablet isn't: it gets the wide screens). */
export function looksLikePhone(userAgent: string, mobileHint?: string | null): boolean {
	if (mobileHint === '?1') return true;
	return /iPhone|iPod|Android.+Mobile|Mobile.+Firefox|Windows Phone/i.test(userAgent);
}

/** The width below which the phone's screens are shown (Tailwind's md). */
export const PHONE_QUERY = '(max-width: 767.98px)';
