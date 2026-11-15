import { describe, expect, it } from 'vitest';
import { DEFAULT_QUIET_HOURS, formatSample, isTime, notificationPreferencesBody, sendsOnEnter } from './accountSettings';

function form(entries: Record<string, string>): FormData {
	const data = new FormData();
	for (const [name, value] of Object.entries(entries)) data.set(name, value);
	return data;
}

describe('notificationPreferencesBody', () => {
	it('sends the switches and quiet hours when they are on', () => {
		expect(
			notificationPreferencesBody(form({ email_account: 'true', email_promos: 'false', push_reminders: 'true', quiet: 'true', quiet_start: '22:00', quiet_end: '06:00' })),
		).toEqual({ email_account: true, email_promos: false, push_reminders: true, quiet_hours: { start: '22:00', end: '06:00' } });
	});

	it('sends null quiet hours when off or not real times', () => {
		expect(notificationPreferencesBody(form({ quiet: 'false', quiet_start: '22:00', quiet_end: '06:00' })).quiet_hours).toBeNull();
		expect(notificationPreferencesBody(form({ quiet: 'true', quiet_start: '25:00', quiet_end: '06:00' })).quiet_hours).toBeNull();
		expect(notificationPreferencesBody(form({ quiet: 'true', quiet_start: '07:00', quiet_end: '07:00' })).quiet_hours).toBeNull();
		expect(notificationPreferencesBody(form({})).push_reminders).toBe(false);
	});

	it('starts from 22:00 to 06:00', () => {
		expect(DEFAULT_QUIET_HOURS).toEqual({ start: '22:00', end: '06:00' });
		expect(isTime('06:00') && !isTime('6:00')).toBe(true);
	});
});

describe('sendsOnEnter', () => {
	const key = (over: Partial<KeyboardEvent> = {}) => ({ key: 'Enter', shiftKey: false, metaKey: false, ctrlKey: false, isComposing: false, ...over });

	it('sends on Enter by default, never with Shift or while composing', () => {
		expect(sendsOnEnter(key(), true)).toBe(true);
		expect(sendsOnEnter(key({ shiftKey: true }), true)).toBe(false);
		expect(sendsOnEnter(key({ isComposing: true }), true)).toBe(false);
		expect(sendsOnEnter(key({ key: 'a' }), true)).toBe(false);
	});

	it('needs Cmd or Ctrl when Enter makes a new line', () => {
		expect(sendsOnEnter(key(), false)).toBe(false);
		expect(sendsOnEnter(key({ metaKey: true }), false)).toBe(true);
		expect(sendsOnEnter(key({ ctrlKey: true }), false)).toBe(true);
	});
});

describe('formatSample', () => {
	it('writes a number and a date the region’s way, or the language’s', () => {
		expect(formatSample('en-US', 'en')).toBe('1,234.5 · 11/14/26');
		expect(formatSample('', 'en-GB')).toBe('1,234.5 · 14/11/2026');
	});
});
