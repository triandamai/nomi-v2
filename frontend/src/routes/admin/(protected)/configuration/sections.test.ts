import { describe, expect, it } from 'vitest';
import { ORDER, SECTIONS } from './sections';

describe('configuration sections', () => {
	it('has a Design section with its switch and rollout, listed on the page', () => {
		expect(ORDER).toContain('design');
		const design = SECTIONS.design;
		expect(design.title()).toBe('Design');
		expect(Object.keys(design.fields)).toEqual(['enabled', 'rollout', 'rollout_allowlist', 'rollout_percent']);
		expect(design.fields.enabled.hint?.()).toMatch(/off/i);
	});

	it('lists only sections it can describe', () => {
		for (const name of ORDER) expect(SECTIONS[name], name).toBeDefined();
	});
});
