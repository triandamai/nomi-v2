// Publishes the static design-system docs (build-gallery/, from `npm run build:gallery`) to
// Cloudflare Pages: main is the live site, any other branch a preview of it. Needs
// CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID, or a `npx wrangler login` on your machine.
//
//   npm run docs:deploy                       # build, then publish as main (the live site)
//   node scripts/docs/deploy.mjs --branch feat/x --sha <commit>
import { spawnSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const WRANGLER = 'wrangler@4.151.1';
// The site's root opens the docs home.
export const REDIRECTS = '/ /design-system 302\n';

/** @param {{ project: string; branch: string; sha?: string }} options */
export function steps({ project, branch, sha }) {
	return [
		{ args: ['pages', 'project', 'create', project, '--production-branch', 'main'], fineIf: /already exists/i },
		{ args: ['pages', 'deploy', 'build-gallery', '--project-name', project, '--branch', branch, ...(sha ? ['--commit-hash', sha] : [])] },
	];
}

/** @param {string[]} argv */
function main(argv) {
	const option = (/** @type {string} */ name) => (argv.includes(name) ? argv[argv.indexOf(name) + 1] : undefined);
	const root = fileURLToPath(new URL('../..', import.meta.url));
	writeFileSync(`${root}build-gallery/_redirects`, REDIRECTS);
	const project = process.env.DOCS_PAGES_PROJECT || 'nomi-design-system';
	for (const step of steps({ project, branch: option('--branch') ?? 'main', sha: option('--sha') })) {
		const run = spawnSync('npx', ['--yes', WRANGLER, ...step.args], { cwd: root, encoding: 'utf8', stdio: ['inherit', 'pipe', 'pipe'] });
		const output = `${run.stdout}${run.stderr}`;
		process.stdout.write(output);
		if (run.status !== 0 && !(step.fineIf && step.fineIf.test(output))) process.exit(run.status ?? 1);
	}
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv.slice(2));
