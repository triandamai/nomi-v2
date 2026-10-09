declare global {
	/** Set at build time from the release tag (see app-version.js). */
	const __APP_VERSION__: string;

	namespace App {
		interface Locals {
			accessToken?: string;
		}
	}
}

export {};
