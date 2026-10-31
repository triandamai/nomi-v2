import { describe, expect, it } from 'vitest';
import { crewId } from './crewId';

describe('crewId', () => {
	it('names the crew member behind an agent_type', () => {
		expect(crewId('coding')).toBe('koda');
		expect(crewId('money')).toBe('dana');
		expect(crewId('reminders')).toBe('kala');
	});

	it('takes a crew name as it is', () => {
		expect(crewId('Koda')).toBe('koda');
		expect(crewId('supervisor')).toBe('supervisor');
	});

	it('falls back to Nomi for chitchat, unknown agents and none', () => {
		expect(crewId('chitchat')).toBe('nomi');
		expect(crewId('inspector')).toBe('nomi');
		expect(crewId(null)).toBe('nomi');
	});
});
