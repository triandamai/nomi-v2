import { describe, expect, it } from 'vitest';
import { initViewport } from './viewport';

describe('initViewport SSR', () => {
	it('returns a noop cleanup function when window is undefined', () => {
		const cleanup = initViewport();
		expect(typeof cleanup).toBe('function');
		expect(() => cleanup()).not.toThrow();
	});
});
