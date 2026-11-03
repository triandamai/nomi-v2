import { describe, expect, it } from 'vitest';
import { initials, peerFor, peerOf, whereLabel } from './peers';

describe('peers', () => {
	it('names people by initials and alternates their colours', () => {
		expect(initials('Ben Tan')).toBe('BT');
		expect(initials('ana')).toBe('AN');
		expect(initials('maria.de.la.cruz')).toBe('MC');
		expect([0, 1, 2].map(peerOf)).toEqual([1, 2, 1]);
		expect(peerFor('Ben Tan')).toBe(peerFor('ben tan'));
		expect([1, 2]).toContain(peerFor('Ana'));
	});
	it('says where someone is', () => {
		const labels = { preview: 'Preview', code: 'Code', terminal: 'Terminal' };
		expect(whereLabel('code', 'src/app.ts', labels)).toBe('Code · src/app.ts');
		expect(whereLabel(null, null, labels)).toBe('Preview');
	});
});
