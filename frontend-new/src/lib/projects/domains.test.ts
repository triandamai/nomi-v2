import { describe, expect, it } from 'vitest';
import { domainView, type OwnDomain } from './domains';

const record = (type: string, status: 'found' | 'waiting') => ({ type, name: type === 'TXT' ? '_nomi-verify' : '@', value: 'x', status });
const domain = (over: Partial<OwnDomain> = {}): OwnDomain => ({
	hostname: 'warungkopibusiti.id',
	records: [record('A', 'waiting'), record('TXT', 'waiting')],
	checked_at: '2026-11-15T01:51:00Z',
	verified_at: null,
	primary: false,
	https: 'pending',
	created_at: '2026-11-15T01:50:00Z',
	...over,
});

describe('a domain of the person’s own', () => {
	it('waits for DNS until every record is found, saying when it was last checked', () => {
		expect(domainView(domain(), 'Asia/Jakarta')).toEqual({
			state: { label: 'waiting for DNS', tone: 'waiting' },
			records: [
				{ type: 'A', name: '@', value: 'x', found: false },
				{ type: 'TXT', name: '_nomi-verify', value: 'x', found: false },
			],
			checked: 'last checked 08:51 · every 5 min',
			note: 'HTTPS is set up automatically once the records are found. DNS can take up to an hour to update.',
			canMakePrimary: false,
		});
	});

	it('is connected once every record is found, with HTTPS on or on its way, and can then be made primary', () => {
		const found = [record('A', 'found'), record('TXT', 'found')];
		const on = domainView(domain({ records: found, verified_at: '2026-11-15T02:14:00Z', https: 'issued' }), 'Asia/Jakarta');
		expect([on.state, on.checked, on.note, on.canMakePrimary]).toEqual([{ label: 'Connected · HTTPS on', tone: 'live' }, 'checked 08:51 · all records found', 'Both addresses serve the live app.', true]);
		const coming = domainView(domain({ records: found, verified_at: '2026-11-15T02:14:00Z' }), 'Asia/Jakarta');
		expect([coming.state, coming.note]).toEqual([{ label: 'Connected · HTTPS on its way', tone: 'live' }, 'Every record is found. HTTPS for this address is still being set up by the hosting.']);
		expect(domainView(domain({ records: found, primary: true }), 'Asia/Jakarta').canMakePrimary).toBe(false);
	});

	it('says it hasn’t been checked yet', () => {
		expect(domainView(domain({ checked_at: null }), 'UTC').checked).toBe('not checked yet · every 5 min');
	});
});
