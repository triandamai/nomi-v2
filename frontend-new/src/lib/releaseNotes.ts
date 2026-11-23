// What changed in the version the app is running, from the GitHub releases CI baked into this
// build (see release-notes.js). Staging and production are told apart by the tag: a staging tag
// ends in -staging (v1.2.3-staging4), as the deploy workflows split them. A build shows only its
// own environment's notes, so production never shows a staging release and the other way round.

export type ReleaseEnvironment = 'staging' | 'production';
export type Release = { version: string; name: string; publishedAt: string; url: string; body: string };
export type ReleaseNotes = { environment: ReleaseEnvironment; releases: Release[] };

/** The notes this build carries; none in a build without them (a local one, usually). */
export const BAKED_RELEASE_NOTES: ReleaseNotes | null = typeof __RELEASE_NOTES__ === 'undefined' ? null : __RELEASE_NOTES__;

const STAGING = /-staging\d*$/;
const PRODUCTION = /^\d+\.\d+\.\d+$/;

/** Where a version was released: staging, production, or neither (a dev build's `git describe`). */
export function environmentOf(version: string): ReleaseEnvironment | null {
	const bare = version.trim().replace(/^v(?=\d)/, '');
	if (STAGING.test(bare)) return 'staging';
	if (PRODUCTION.test(bare)) return 'production';
	return null;
}

/**
 * The releases a build of `version` shows: only its own environment's, newest first, starting at
 * its own (anything newer isn't in this build).
 */
export function releasesFor(notes: ReleaseNotes | null, version: string): Release[] {
	const environment = environmentOf(version);
	if (!notes || !environment || notes.environment !== environment) return [];
	const own = notes.releases.filter((release) => environmentOf(release.version) === environment);
	const at = own.findIndex((release) => release.version === version);
	return at > 0 ? own.slice(at) : own;
}

/** Remembers, per browser, the last version whose notes the person saw. */
export const SEEN_KEY = 'nomi.release-notes.seen';

/** Whether the popup should open for `version`: it has notes, and the person hasn't seen them. */
export function shouldShow(releases: { version: string }[], version: string, storage: Pick<Storage, 'getItem'> | undefined): boolean {
	if (!releases.some((release) => release.version === version)) return false;
	try {
		return storage?.getItem(SEEN_KEY) !== version;
	} catch {
		// Storage blocked (a private window): showing it each load is better than never.
		return true;
	}
}

export function markSeen(version: string, storage: Pick<Storage, 'setItem'> | undefined): void {
	try {
		storage?.setItem(SEEN_KEY, version);
	} catch {
		// Storage blocked: nothing to remember it in.
	}
}
