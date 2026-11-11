import { describe, expect, it } from 'vitest';
import { keep, take, waiting, type OutboxStorage } from './outbox';

function memory(): OutboxStorage {
	const items = new Map<string, string>();
	return {
		getItem: (key) => items.get(key) ?? null,
		setItem: (key, value) => void items.set(key, value),
		removeItem: (key) => void items.delete(key),
	};
}

const blocked: OutboxStorage = {
	getItem: () => {
		throw new Error('blocked');
	},
	setItem: () => {
		throw new Error('blocked');
	},
	removeItem: () => {
		throw new Error('blocked');
	},
};

describe('outbox', () => {
	it('keeps one message per chat until it is taken', () => {
		const storage = memory();
		keep('/chat/a', 'first', storage);
		keep('/chat/a', 'second', storage);
		keep('/chat/b', 'other', storage);
		expect(waiting('/chat/a', storage)).toBe(true);
		expect(take('/chat/a', storage)).toBe('second');
		expect(take('/chat/a', storage)).toBeNull();
		expect(waiting('/chat/a', storage)).toBe(false);
		expect(take('/chat/b', storage)).toBe('other');
	});

	it('does without storage', () => {
		expect(() => keep('/chat/a', 'hi', blocked)).not.toThrow();
		expect(waiting('/chat/a', blocked)).toBe(false);
		expect(take('/chat/a', blocked)).toBeNull();
	});
});
