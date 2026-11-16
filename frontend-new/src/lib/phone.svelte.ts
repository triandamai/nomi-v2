import { MediaQuery } from 'svelte/reactivity';
import { PHONE_QUERY } from './phone';

/**
 * Whether to show the phone's screens: the window's width once in the browser, the server's guess
 * (`looksLikePhone`) until then, so the first paint already matches.
 */
export function phoneView(guess: boolean): { readonly current: boolean } {
	return new MediaQuery(PHONE_QUERY, guess);
}
