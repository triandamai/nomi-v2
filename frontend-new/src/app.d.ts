// See https://svelte.dev/docs/kit/types#app.d.ts
declare global {
	/** Set at build time from the release tag (see app-version.js). */
	const __APP_VERSION__: string;
	/** True in `npm run build:gallery`, the static copy of the design-system gallery. */
	const __DS_STATIC__: boolean;
	/** The release notes CI baked into this build (see release-notes.js); null without them. */
	const __RELEASE_NOTES__: import('$lib/releaseNotes').ReleaseNotes | null;

	namespace App {
		interface Locals {
			/** The person's access token for this request (see $lib/server/api). */
			accessToken?: string;
			/** The one sign-in renewal this request makes, shared by all its calls. */
			refreshing?: Promise<string | null>;
		}
	}
}

export {};
