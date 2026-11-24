import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { test as base, expect } from '@playwright/test';

const coverageDir = process.env.E2E_COVERAGE === '1' ? (process.env.E2E_COVERAGE_DIR ?? '.e2e-coverage/default') : null;

export const test = base.extend<{ _coverage: void }>({
	_coverage: [
		async ({ page }, use, testInfo) => {
			if (!coverageDir) {
				await use();
				return;
			}
			await page.coverage.startJSCoverage({ resetOnNavigation: false });
			try {
				await use();
			} finally {
				try {
					const raw = await page.coverage.stopJSCoverage();
					const result = raw
						.filter((entry) => entry.url.includes('/_app/immutable/'))
						.map(({ url, functions }) => ({ url, functions }));
					if (result.length > 0) {
						const dir = join(coverageDir, 'client');
						await mkdir(dir, { recursive: true });
						const id = `${testInfo.workerIndex}-${testInfo.testId.replace(/[^a-zA-Z0-9_-]/g, '_')}-${testInfo.retry}.json`;
						await writeFile(join(dir, id), JSON.stringify({ result }));
					}
				} catch {
					// Page may already be closed on aborted tests; skip coverage for this test.
				}
			}
		},
		{ auto: true },
	],
});

export { expect };
export type { BrowserContext, Locator, Page } from '@playwright/test';
