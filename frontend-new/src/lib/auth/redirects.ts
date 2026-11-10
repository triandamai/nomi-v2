// Where someone is going after signing in travels with them, from page to page, as `redirect_to`.

const INVITE = /^\/invite\/([A-Za-z0-9_-]+)(?:[?#].*)?$/;

/** The invite code when a sign-up or sign-in is on its way to `/invite/:code`, else null. */
export function inviteCodeFrom(target: string | null | undefined): string | null {
	return target?.match(INVITE)?.[1] ?? null;
}

/** Only paths on this site, and not the home page (which is where everyone goes anyway). */
function worthKeeping(target: string | null | undefined): target is string {
	return Boolean(target) && target!.startsWith('/') && !target!.startsWith('//') && target !== '/';
}

/** `path`, carrying `redirectTo` (and any other non-empty values) in its query. */
export function withRedirect(path: string, redirectTo: string | null | undefined, extra: Record<string, string | null | undefined> = {}): string {
	const query = new URLSearchParams();
	if (worthKeeping(redirectTo)) query.set('redirect_to', redirectTo);
	for (const [key, value] of Object.entries(extra)) if (value) query.set(key, value);
	const search = query.toString();
	return search ? `${path}?${search}` : path;
}
