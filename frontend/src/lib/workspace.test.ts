import { describe, expect, it } from 'vitest';
import { connectLink, serviceLabel } from './workspace';

describe('workspace', () => {
	it('links to the connect sheet with the needed services and the chat to resume', () => {
		expect(connectLink(['sheets', 'photos'], 'abc')).toBe('/connections?connect=1&services=sheets&resume=abc');
		expect(connectLink([])).toBe('/connections?connect=1');
	});

	it('names known services and leaves others as they are', () => {
		expect(serviceLabel('gmail')).toBe('Gmail');
		expect(serviceLabel('photos')).toBe('photos');
	});
});
