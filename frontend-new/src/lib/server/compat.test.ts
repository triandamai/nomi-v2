import { describe, expect, it } from 'vitest';
import { legacyPath } from './compat';

// The backend writes links to these paths into emails and notifications (APP_URL + path), the
// old app's addresses: v2 sends each to its own page.
describe('legacyPath', () => {
	it('maps every address the backend links to onto its v2 page, keeping the query', () => {
		expect(legacyPath({ pathname: '/login', search: '?redirect_to=%2Fchat' })).toBe('/sign-in?redirect_to=%2Fchat');
		expect(legacyPath({ pathname: '/organizations/9b2c', search: '' })).toBe('/settings/workspaces/9b2c');
		expect(legacyPath({ pathname: '/projects/p1/app', search: '?sheet=deploy' })).toBe('/projects/p1?sheet=deploy');
		expect(legacyPath({ pathname: '/billing', search: '' })).toBe('/settings/plan');
		expect(legacyPath({ pathname: '/models', search: '' })).toBe('/settings/models');
	});

	it('leaves every other path alone', () => {
		for (const path of ['/', '/sign-in', '/projects/p1', '/organizations', '/login/extra']) expect(legacyPath({ pathname: path, search: '' })).toBeNull();
	});

	it('reads the query only for an address it maps (prerendered pages may not read it)', () => {
		const url = {
			pathname: '/design-system',
			get search(): string {
				throw new Error('Cannot access url.search on a page with prerendering enabled');
			},
		};
		expect(legacyPath(url)).toBeNull();
	});
});
