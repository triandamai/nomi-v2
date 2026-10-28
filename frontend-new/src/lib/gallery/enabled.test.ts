import { describe, expect, it } from 'vitest';
import { galleryEnabled } from './enabled';

// The design-system gallery is a tool for building and reviewing components: on in development
// and CI, off in the production image.

describe('galleryEnabled', () => {
	it('is on only when PUBLIC_DS_GALLERY says so', () => {
		expect(galleryEnabled('1')).toBe(true);
		expect(galleryEnabled('true')).toBe(true);
		expect(galleryEnabled(undefined)).toBe(false);
		expect(galleryEnabled('')).toBe(false);
		expect(galleryEnabled('0')).toBe(false);
		expect(galleryEnabled('false')).toBe(false);
	});
});
