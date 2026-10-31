import { describe, expect, it } from 'vitest';
import { activeSetting, filterSettings, settingsGroups } from './settingsNav';

describe('settings drawer', () => {
	it('knows a settings page, and the pages below it, by path', () => {
		expect(activeSetting('/billing')).toBe('/billing');
		expect(activeSetting('/connections/github')).toBe('/connections');
		expect(activeSetting('/chats')).toBeUndefined();
		expect(activeSetting('/profiles')).toBeUndefined();
	});

	it('lists every settings page once', () => {
		const hrefs = settingsGroups().flatMap((group) => group.items.map((item) => item.href));
		expect(new Set(hrefs).size).toBe(hrefs.length);
		for (const href of hrefs) expect(activeSetting(href)).toBe(href);
	});

	it('searches by name in any case, dropping groups left empty', () => {
		const found = filterSettings(settingsGroups(), '  BILL ');
		expect(found).toHaveLength(1);
		expect(found[0].items.map((item) => item.href)).toEqual(['/billing']);
		expect(filterSettings(settingsGroups(), '')).toEqual(settingsGroups());
		expect(filterSettings(settingsGroups(), 'nothing like this')).toEqual([]);
	});
});
