import { describe, expect, it } from 'vitest';
import { distinctInitials, initials, peerFor, pointersIn, whereLabel } from './peers';

describe('people in a shared project', () => {
	it('names people by initials, each in one of the two peer colours for good', () => {
		expect(initials('Sari Wulandari')).toBe('SW');
		expect(initials('ana')).toBe('AN');
		expect(initials('maria.de.la.cruz')).toBe('MC');
		expect(initials('  ')).toBe('?');
		expect(peerFor('u-1')).toBe(peerFor('u-1'));
		expect([1, 2]).toContain(peerFor('u-2'));
		expect(new Set(['a', 'b', 'c', 'd', 'e'].map(peerFor)).size).toBe(2);
	});

	it('keeps everyone’s initials apart, so two people never share a face', () => {
		expect(distinctInitials(['Mario Prasetyo', 'Sari Wulandari'])).toEqual(['MP', 'SW']);
		expect(distinctInitials(['Mario Prasetyo', 'Maya Putri', 'Mira Pane'])).toEqual(['MP', 'MA', 'MI']);
		expect(distinctInitials(['Ana', 'ana', 'Ana'])).toEqual(['AN', 'AN2', 'AN3']);
	});

	it('says where someone is', () => {
		const labels = { preview: 'Preview', code: 'Code', terminal: 'Terminal' };
		expect(whereLabel('code', 'src/app.ts', labels)).toBe('Code · src/app.ts');
		expect(whereLabel('code', null, labels)).toBe('Code');
		expect(whereLabel('terminal', 'x', labels)).toBe('Terminal');
		expect(whereLabel(null, null, labels)).toBe('Preview');
	});

	it('shows the pointers over one view that moved in the last few seconds', () => {
		const pointers = { a: { name: 'Sari', x: 0.2, y: 0.5, view: 'preview', at: 10_000 }, b: { name: 'Ben', x: 0.1, y: 0.1, view: 'code', at: 10_000 }, c: { name: 'Ana', x: 0.9, y: 0.9, view: 'preview', at: 1_000 } };
		expect(pointersIn(pointers, 'preview', 12_000)).toEqual([{ user_id: 'a', name: 'Sari', x: 0.2, y: 0.5, view: 'preview', at: 10_000 }]);
		expect(pointersIn(pointers, 'terminal', 12_000)).toEqual([]);
	});
});
