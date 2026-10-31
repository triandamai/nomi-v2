import { describe, expect, it } from 'vitest';
import { REDIRECTS, WRANGLER, steps } from '../../../scripts/docs/deploy.mjs';

describe('publishing the docs to Cloudflare Pages', () => {
	it('makes the project once, with main as the live site, then deploys the branch', () => {
		expect(steps({ project: 'nomi-design-system', branch: 'main', sha: 'abc123' })).toEqual([
			{ args: ['pages', 'project', 'create', 'nomi-design-system', '--production-branch', 'main'], fineIf: /already exists/i },
			{ args: ['pages', 'deploy', 'build-gallery', '--project-name', 'nomi-design-system', '--branch', 'main', '--commit-hash', 'abc123'] },
		]);
	});

	it('deploys any other branch as a preview, and works without a commit', () => {
		expect(steps({ project: 'p', branch: 'feat/x' })[1].args).toEqual(['pages', 'deploy', 'build-gallery', '--project-name', 'p', '--branch', 'feat/x']);
	});

	it('opens the site on the docs home, with a pinned Wrangler', () => {
		expect(REDIRECTS).toBe('/ /design-system 302\n');
		expect(WRANGLER).toMatch(/^wrangler@\d+\.\d+\.\d+$/);
	});
});
