import { join, sep } from 'node:path';
import { describe, expect, it } from 'vitest';
import { outsideDocs } from '../../../vite-plugins/docs-only-routes';

const routes = join('/app', 'src', 'routes') + sep;
const at = (...parts: string[]) => join(routes, ...parts);

describe('the static docs leave out the app’s own routes', () => {
	it('stubs every route outside design-system, keeping the docs and the root layout', () => {
		expect(outsideDocs(at('(app)', 'projects', '+page.svelte'), routes)).toBe(true);
		expect(outsideDocs(at('admin', '+layout.server.ts'), routes)).toBe(true);
		expect(outsideDocs(at('hooks', 'sites', '[projectId]', '+server.ts'), routes)).toBe(true);
		expect(outsideDocs(at('design-system', 'components', '[slug]', '+page.svelte'), routes)).toBe(false);
		expect(outsideDocs(at('+layout.svelte'), routes)).toBe(false);
		expect(outsideDocs(join('/app', 'src', 'lib', 'ds', 'actions', 'Button.svelte'), routes)).toBe(false);
	});
});
