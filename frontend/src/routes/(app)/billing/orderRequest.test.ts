import { describe, expect, it } from 'vitest';
import { readOrderRequest } from './orderRequest';

describe('readOrderRequest', () => {
	it('reads a plan, months and an optional trimmed code', () => {
		expect(readOrderRequest({ plan_id: 'p1', months: 6 })).toEqual({ plan_id: 'p1', months: 6 });
		expect(readOrderRequest({ plan_id: 'p1', months: 6, code: ' SAVE20 ' })).toEqual({ plan_id: 'p1', months: 6, code: 'SAVE20' });
		expect(readOrderRequest({ plan_id: 'p1', months: 1, code: '   ' })).toEqual({ plan_id: 'p1', months: 1 });
	});

	it('refuses anything else', () => {
		expect(readOrderRequest(null)).toBeNull();
		expect(readOrderRequest('x')).toBeNull();
		expect(readOrderRequest({ months: 6 })).toBeNull();
		expect(readOrderRequest({ plan_id: 'p1', months: 0 })).toBeNull();
		expect(readOrderRequest({ plan_id: 'p1', months: '6' })).toBeNull();
		expect(readOrderRequest({ plan_id: 'p1', months: 1.5 })).toBeNull();
	});
});
