import { describe, expect, it } from 'vitest';
import { expireTyping, isBackendEvent, isDeployEvent, shouldSend, typingNames } from './presenceView';

describe('typingNames', () => {
	it('lists who is typing, the first to start first, without the person themself', () => {
		// Ben started later but pinged last: he still comes second.
		const typing = { u2: { name: 'Ben', since: 2_000, at: 9_000 }, u3: { name: 'Cy', since: 1_000, at: 5_000 }, me: { name: 'Ana', since: 500, at: 3_000 } };
		expect(typingNames(typing, 'me')).toEqual(['Cy', 'Ben']);
		expect(typingNames({}, 'me')).toEqual([]);
	});
});

describe('expireTyping', () => {
	it('drops anyone whose last ping is older than the time to live', () => {
		const typing = { u2: { name: 'Ben', since: 1_000, at: 1_000 }, u3: { name: 'Cy', since: 2_000, at: 9_000 } };
		expect(Object.keys(expireTyping(typing, 10_000, 6_000))).toEqual(['u3']);
	});
});

describe('withPing', () => {
	it('keeps when someone started and refreshes their last ping', async () => {
		const { withPing } = await import('./presenceView');
		const once = withPing({}, 'u2', 'Ben', 1_000);
		const twice = withPing(once, 'u2', 'Ben', 4_000);
		expect(twice.u2).toEqual({ name: 'Ben', since: 1_000, at: 4_000 });
	});
});

describe('shouldSend', () => {
	it('sends at most one ping every interval', () => {
		expect(shouldSend(null, 5_000, 3_000)).toBe(true);
		expect(shouldSend(4_000, 5_000, 3_000)).toBe(false);
		expect(shouldSend(1_000, 5_000, 3_000)).toBe(true);
	});
});

describe('isDeployEvent', () => {
	it('tells publishing changes apart from presence and typing', () => {
		expect(isDeployEvent('{"kind":"DeployChanged","event":"publish.succeeded"}')).toBe(true);
		expect(isDeployEvent('{"kind":"PresenceChanged"}')).toBe(false);
		expect(isDeployEvent('{"kind":"Typing","user_id":"u2","name":"Ben"}')).toBe(false);
		// Anything this build can't read is treated as a change to be safe.
		expect(isDeployEvent('not json')).toBe(true);
	});
});

describe('isBackendEvent', () => {
	it('is true only for a change to the project backend', () => {
		expect(isBackendEvent('{"kind":"DeployChanged","event":"backend.changed"}')).toBe(true);
		expect(isBackendEvent('{"kind":"DeployChanged","event":"publish.succeeded"}')).toBe(false);
		expect(isBackendEvent('not json')).toBe(false);
	});
});
