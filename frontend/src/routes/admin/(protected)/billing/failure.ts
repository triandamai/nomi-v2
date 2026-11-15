import { fail } from '@sveltejs/kit';

/** A backend refusal as a form failure: the backend's own words (they are plain text), else `fallback`. */
export async function failure(response: Response, fallback: string) {
	const message = (await response.text()).trim();
	return fail(response.status, { error: message || fallback });
}
