import { afterEach, describe, expect, it } from 'vitest';
import { THEME_COLOR } from '$lib/seo/seo';
import { applyTheme } from './theme';

function themeColorTags() {
	return [...document.head.querySelectorAll<HTMLMetaElement>('meta[name="theme-color"]')].map((tag) => ({
		content: tag.content,
		media: tag.media || null,
	}));
}

describe('applyTheme', () => {
	afterEach(() => {
		applyTheme('system');
	});

	it('updates data-theme and replaces theme-color meta tags so Safari repaints its chrome', () => {
		applyTheme('dark');
		expect(document.documentElement.dataset.theme).toBe('dark');
		expect(themeColorTags()).toEqual([{ content: THEME_COLOR.dark, media: null }]);

		applyTheme('light');
		expect(document.documentElement.dataset.theme).toBe('light');
		expect(themeColorTags()).toEqual([{ content: THEME_COLOR.light, media: null }]);

		applyTheme('system');
		expect(document.documentElement.dataset.theme).toBeUndefined();
		expect(themeColorTags()).toEqual([
			{ content: THEME_COLOR.light, media: '(prefers-color-scheme: light)' },
			{ content: THEME_COLOR.dark, media: '(prefers-color-scheme: dark)' },
		]);
	});
});
