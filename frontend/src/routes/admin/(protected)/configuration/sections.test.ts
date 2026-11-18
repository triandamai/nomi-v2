import { describe, expect, it } from 'vitest';
import { ORDER, SECTIONS } from './sections';

describe('configuration sections', () => {
	it('has no Design section: Sora is switched in Admin → Agents like every other agent', () => {
		expect(ORDER).not.toContain('design');
		expect(SECTIONS.design).toBeUndefined();
	});

	it('lists only sections it can describe', () => {
		for (const name of ORDER) expect(SECTIONS[name], name).toBeDefined();
	});
});
