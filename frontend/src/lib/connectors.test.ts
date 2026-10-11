import { describe, expect, it } from 'vitest';
import { CONNECTORS, searchConnectors } from './connectors';

describe('searchConnectors', () => {
	it('finds connectors by name, what they do and their category', () => {
		expect(searchConnectors(CONNECTORS, 'hevy').map((c) => c.id)).toEqual(['hevy']);
		expect(searchConnectors(CONNECTORS, 'GMAIL').map((c) => c.id)).toEqual(['google']);
		expect(searchConnectors(CONNECTORS, 'health').map((c) => c.id)).toEqual(['hevy', 'samsung_health', 'apple_health']);
		expect(searchConnectors(CONNECTORS, '  ')).toHaveLength(CONNECTORS.length);
		expect(searchConnectors(CONNECTORS, 'nothing like this')).toEqual([]);
	});
});
