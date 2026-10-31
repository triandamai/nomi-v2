import { describe, expect, it } from 'vitest';
import { describeBrowser } from './device';

describe('describeBrowser', () => {
	it('names Chrome on a Mac', () => {
		const ua = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36';
		expect(describeBrowser(ua)).toEqual({ name: 'Chrome on macOS', brand: 'Google', model: 'Chrome 130', os_name: 'macOS', os_version: '10.15.7' });
	});

	it('tells Edge from the Chrome it is built on', () => {
		const ua = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36 Edg/130.0.2849.56';
		expect(describeBrowser(ua)).toMatchObject({ name: 'Edge on Windows', brand: 'Microsoft', os_version: '10.0' });
	});

	it('names Safari on an iPhone', () => {
		const ua = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1';
		expect(describeBrowser(ua)).toMatchObject({ name: 'Safari on iOS', brand: 'Apple', model: 'Safari 18', os_version: '18.1' });
	});

	it('names Firefox on Android', () => {
		const ua = 'Mozilla/5.0 (Android 15; Mobile; rv:132.0) Gecko/132.0 Firefox/132.0';
		expect(describeBrowser(ua)).toMatchObject({ name: 'Firefox on Android', os_version: '15' });
	});

	it('still says something for an agent it does not know', () => {
		expect(describeBrowser('curl/8.4.0')).toEqual({ name: 'Browser', brand: null, model: null, os_name: null, os_version: null });
	});
});
