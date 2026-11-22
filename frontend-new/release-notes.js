/**
 * The release notes baked into this build. CI (release.yml, staging.yml) reads the GitHub releases
 * of the build's own environment, newest first, and passes them as `RELEASE_NOTES`: JSON, base64
 * encoded so it fits on one build-arg line. Raw JSON works too (handy locally). Without it, or if
 * it can't be read, the build has no notes and the app never shows the popup.
 *
 * Shape: `{ environment: 'staging' | 'production', releases: [{ version, name, publishedAt, url, body }] }`.
 * A leading `v` is dropped from each version, as app-version.js drops it from the app's own.
 * @returns {{ environment: 'staging' | 'production', releases: { version: string, name: string, publishedAt: string, url: string, body: string }[] } | null}
 */
export function releaseNotes() {
	const raw = process.env.RELEASE_NOTES?.trim();
	if (!raw) return null;
	try {
		const text = raw.startsWith('{') ? raw : Buffer.from(raw, 'base64').toString('utf8');
		return normalise(JSON.parse(text));
	} catch {
		console.warn('RELEASE_NOTES is set but is not JSON (or base64 JSON); building without release notes.');
		return null;
	}
}

/** The most releases a build carries: the one it is plus a few before it. */
const MAX_RELEASES = 10;

const ENVIRONMENTS = new Set(['staging', 'production']);

/** @param {unknown} value */
const text = (value) => (typeof value === 'string' ? value : '');

/** @param {any} payload */
function normalise(payload) {
	if (!payload || !ENVIRONMENTS.has(payload.environment) || !Array.isArray(payload.releases)) return null;
	const releases = payload.releases
		.map((/** @type {any} */ release) => ({
			version: text(release?.version).trim().replace(/^v(?=\d)/, ''),
			name: text(release?.name).trim(),
			publishedAt: text(release?.publishedAt),
			url: text(release?.url),
			// GitHub's generated notes come with Windows line endings.
			body: text(release?.body).replace(/\r\n/g, '\n').trim(),
		}))
		.filter((/** @type {{ version: string }} */ release) => release.version)
		.slice(0, MAX_RELEASES);
	return { environment: payload.environment, releases };
}
