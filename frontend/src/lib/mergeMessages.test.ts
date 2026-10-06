import { describe, expect, it } from 'vitest';
import { mergeReloaded } from './mergeMessages';

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
