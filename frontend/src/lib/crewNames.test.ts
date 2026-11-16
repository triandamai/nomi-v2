import { describe, expect, it } from 'vitest';
import { CREW_TYPE_BY_NAME, crewName } from './crewNames';

describe('crew names', () => {
	it('calls the design agent Sora, both ways', () => {
		expect(crewName('design')).toBe('Sora');
		expect(CREW_TYPE_BY_NAME.sora).toBe('design');
	});

	it('keeps the rest of the crew and falls back to the type', () => {
		expect(crewName('planning')).toBe('Rena');
		expect(crewName('coding')).toBe('Koda');
		expect(crewName('something-else')).toBe('something-else');
	});
});
