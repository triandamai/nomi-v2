import { describe, expect, it } from 'vitest';
import { canMakePrimary, foundCount, recordHost, typedHostname } from './customDomains';
import type { CustomDomain } from '$lib/types';

const domain = (over: Partial<CustomDomain> = {}): CustomDomain => ({
	hostname: 'shop.example.com',
	records: [
		{ type: 'CNAME', name: 'shop.example.com', value: 'todo.apps.nomi.app', status: 'found' },
		{ type: 'TXT', name: '_nomi-verify.shop.example.com', value: 'nomi-verify=abc', status: 'waiting' },
	],
	checked_at: '2026-11-15T09:00:00Z',
	verified_at: null,
	primary: false,
	https: 'pending',
	created_at: '2026-11-15T08:00:00Z',
	...over,
});

describe('custom domains', () => {
	it('counts the records found', () => {
		expect(foundCount(domain())).toEqual({ found: 1, total: 2 });
	});

	it('makes only a verified, non-primary domain primary', () => {
		expect(canMakePrimary(domain())).toBe(false);
		expect(canMakePrimary(domain({ verified_at: '2026-11-15T09:00:00Z' }))).toBe(true);
		expect(canMakePrimary(domain({ verified_at: '2026-11-15T09:00:00Z', primary: true }))).toBe(false);
	});

	it('reads a typed hostname like the backend', () => {
		expect(typedHostname(' HTTPS://Shop.Example.com/menu ')).toBe('shop.example.com');
		expect(typedHostname('example.com.')).toBe('example.com');
	});

	it('names records as a registrar does', () => {
		expect(recordHost('example.com', 'example.com')).toBe('@');
		expect(recordHost('_nomi-verify.example.com', 'example.com')).toBe('_nomi-verify');
		expect(recordHost('_nomi-verify.shop.example.com', 'shop.example.com')).toBe('_nomi-verify');
	});
});
