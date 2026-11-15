import { describe, expect, it } from 'vitest';
import { deviceName, devicesLine, maskEmail, parseQuiet, quietOptions, quietValue, zoneOptions } from './general';

const device = (id: string, extra: Record<string, unknown> = {}) => ({
	id,
	platform: 'web',
	name: null,
	brand: null,
	model: null,
	os_name: null,
	os_version: null,
	app_version: '1.0',
	presence: 'offline',
	last_seen_at: '2026-10-21T01:00:00Z',
	created_at: '2026-10-01T00:00:00Z',
	current: false,
	...extra,
});

describe('Settings › General’s wording', () => {
	it('hides most of the email: the first two letters, then dots, then the domain', () => {
		expect(maskEmail('kotaksurat@gmail.com')).toBe('ko••••••••@gmail.com');
		expect(maskEmail('ab@x.id')).toBe('ab•••@x.id');
	});

	it('names a device by what it says it is', () => {
		expect(deviceName(device('d', { name: 'MacBook Air' }))).toBe('MacBook Air');
		expect(deviceName(device('d', { brand: 'Apple', model: 'iPhone 15' }))).toBe('Apple iPhone 15');
		expect(deviceName(device('d', { os_name: 'Android' }))).toBe('Android');
		expect(deviceName(device('d'))).toBe('web');
	});

	it('lists this device first, then the others with when they were last seen', () => {
		const now = new Date('2026-10-21T03:00:00Z');
		expect(devicesLine([device('a', { model: 'iPhone 15' }), device('b', { name: 'MacBook Air', current: true })], now)).toBe('MacBook Air (this one) · iPhone 15 · 2h');
	});

	it('offers every zone Intl knows, named with its short name', () => {
		const zones = zoneOptions(new Date('2026-10-21T03:00:00Z'));
		expect(zones.length).toBeGreaterThan(300);
		expect(zones.find((zone) => zone.value === 'Asia/Jakarta')?.label).toMatch(/^Asia\/Jakarta · (WIB|GMT\+7)$/);
	});
});

describe('quiet hours', () => {
	it('offers the usual nights, with a saved range of its own kept first', () => {
		expect(quietOptions(null).map((option) => option.label)).toEqual(['21:00–06:00', '22:00–06:00', '22:00–07:00', '23:00–07:00', '00:00–08:00']);
		expect(quietOptions({ start: '22:00', end: '06:00' })).toHaveLength(5);
		expect(quietOptions({ start: '20:30', end: '05:15' })[0]).toEqual({ value: '20:30-05:15', label: '20:30–05:15' });
	});

	it('reads a choice back into the backend’s start and end, refusing anything else', () => {
		expect(quietValue({ start: '22:00', end: '06:00' })).toBe('22:00-06:00');
		expect(parseQuiet('23:00-07:00')).toEqual({ start: '23:00', end: '07:00' });
		for (const bad of ['', 'off', '25:00-06:00', '22:00-22:00', '22:00–06:00', '22:00-6:00']) expect(parseQuiet(bad)).toBeNull();
	});
});

