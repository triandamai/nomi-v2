// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { keep, take, waiting } from './outbox';

afterEach(() => {
	localStorage.clear();
	vi.restoreAllMocks();
});

describe('outbox', () => {
	it('keeps one message per chat until it is taken', () => {
		keep('/chat/a', 'first');
		keep('/chat/a', 'second');
		keep('/chat/b', 'other');
		expect(waiting('/chat/a')).toBe(true);
		expect(take('/chat/a')).toBe('second');
		expect(take('/chat/a')).toBeNull();
		expect(waiting('/chat/a')).toBe(false);
		expect(take('/chat/b')).toBe('other');
	});

	it('does without storage', () => {
		vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
			throw new Error('blocked');
		});
		vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
			throw new Error('blocked');
		});
		expect(() => keep('/chat/a', 'hi')).not.toThrow();
		expect(waiting('/chat/a')).toBe(false);
		expect(take('/chat/a')).toBeNull();
	});
});
