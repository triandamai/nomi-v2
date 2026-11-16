import { describe, expect, it } from 'vitest';
import { looksLikePhone } from './phone';

describe('looking like a phone', () => {
	it('is a phone browser, or one that says it’s mobile; a tablet or a desktop isn’t', () => {
		const iphone = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
		const android = 'Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0 Mobile Safari/537.36';
		const tablet = 'Mozilla/5.0 (Linux; Android 14; SM-X710) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0 Safari/537.36';
		const ipad = 'Mozilla/5.0 (iPad; CPU OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
		const mac = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0 Safari/537.36';
		expect([iphone, android, tablet, ipad, mac].map((agent) => looksLikePhone(agent))).toEqual([true, true, false, false, false]);
		expect(looksLikePhone(mac, '?1')).toBe(true);
	});
});
