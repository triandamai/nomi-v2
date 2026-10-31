import { describe, expect, it } from 'vitest';
import { routeOf } from './static-path';

describe('routeOf', () => {
	it("reads a static copy's .html page as its route, wherever it's hosted", () => {
		expect(routeOf('/shared/nomi-ds/design-system/components/segmented.html')).toBe('/shared/nomi-ds/design-system/components/segmented');
		expect(routeOf('/design-system.html')).toBe('/design-system');
	});

	it('leaves other paths alone', () => {
		expect(routeOf('/design-system/components/segmented')).toBe('/design-system/components/segmented');
	});
});
