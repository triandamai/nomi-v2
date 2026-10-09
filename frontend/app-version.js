import { execSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

/**
 * The version this build of the frontend is. A release build gets it from its tag (CI passes
 * `APP_VERSION=v1.2.3`); anywhere else it's `git describe` (e.g. `0.0.15-68-g419e492`), or
 * package.json's version plus `-dev` without git. A leading `v` is dropped.
 * @returns {string}
 */
export function appVersion() {
	const clean = (/** @type {string} */ v) => v.trim().replace(/^v(?=\d)/, '');
	const fromEnv = process.env.APP_VERSION?.trim();
	if (fromEnv) return clean(fromEnv);
	try {
		const described = execSync('git describe --tags --always --dirty', { stdio: ['ignore', 'pipe', 'ignore'] }).toString();
		if (described.trim()) return clean(described);
	} catch {
		// No git (e.g. a Docker build without APP_VERSION): fall through.
	}
	const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'));
	return `${pkg.version}-dev`;
}
