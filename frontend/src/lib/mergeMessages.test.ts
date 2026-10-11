import { describe, expect, it } from 'vitest';
import { mergeReloaded, prependOlder } from './mergeMessages';

const msg = (id: string, at: string) => ({ id, created_at: `2026-10-06T15:${at}Z` });

describe('mergeReloaded', () => {
	it('keeps a message that arrived live after the reload snapshot was taken', () => {
		const incoming = [msg('ask', '53:00')];
		const shown = [msg('ask', '53:00'), msg('failure-notice', '53:01')];
		expect(mergeReloaded(incoming, shown).map((m) => m.id)).toEqual(['ask', 'failure-notice']);
	});

	it('takes the reloaded copy of a message it already shows', () => {
		const incoming = [{ ...msg('reply', '53:00'), text: 'new' }];
		const shown = [{ ...msg('reply', '53:00'), text: 'old' }];
		expect(mergeReloaded(incoming, shown)).toEqual(incoming);
	});

	it('drops older messages the reload no longer has (deleted, or another chat)', () => {
		const incoming = [msg('b', '53:05')];
		const shown = [msg('a', '53:00'), msg('b', '53:05')];
		expect(mergeReloaded(incoming, shown).map((m) => m.id)).toEqual(['b']);
	});
});

describe('earlier pages', () => {
	const msg = (id: string, created_at: string) => ({ id, created_at });

	it('keeps earlier pages when the newest page reloads', () => {
		const shown = [msg('a', '01'), msg('b', '02'), msg('c', '03'), msg('d', '04')];
		const reloaded = [msg('c', '03'), msg('d', '04'), msg('e', '05')];
		expect(mergeReloaded(reloaded, shown, true).map((m) => m.id)).toEqual(['a', 'b', 'c', 'd', 'e']);
	});

	it('puts an earlier page in front without repeats', () => {
		expect(prependOlder([msg('a', '01'), msg('b', '02')], [msg('b', '02'), msg('c', '03')]).map((m) => m.id)).toEqual(['a', 'b', 'c']);
	});
});
