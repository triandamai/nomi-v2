import { expect, test, type Page } from '@playwright/test';

// The production server (server.js: SvelteKit + the live-update relay) against a stub backend
// (e2e/stub-api.mjs), in a real browser.

/** Opens the person's activity feed from the page and reports how it went. */
function openActivityFeed(page: Page): Promise<{ opened: boolean; message?: string; code?: number }> {
	return page.evaluate(
		() =>
			new Promise((resolve) => {
				const socket = new WebSocket(`${location.origin.replace('http', 'ws')}/activity/ws`);
				let opened = false;
				socket.onopen = () => (opened = true);
				socket.onmessage = (event) => {
					resolve({ opened, message: String(event.data) });
					socket.close();
				};
				socket.onclose = (event) => resolve({ opened, code: event.code });
			}),
	);
}

test('serves the app in the person’s language and theme from the first byte', async ({ page, context }) => {
	await context.addCookies([{ name: 'nm_theme', value: 'dark', url: 'http://localhost:4175' }]);
	const response = await page.goto('/');
	expect(response?.headers()['cache-control']).toBe('no-cache');
	const html = await response!.text();
	expect(html).toMatch(/<html lang="en" data-theme="dark">/);
	await expect(page).toHaveTitle('Nomi');
});

test('refuses live updates to someone signed out', async ({ page }) => {
	await page.goto('/');
	expect(await openActivityFeed(page)).toEqual({ opened: true, code: 4401 });
});

test('relays live updates to someone signed in', async ({ page, context }) => {
	await context.addCookies([{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' }]);
	await page.goto('/');
	expect(await openActivityFeed(page)).toEqual({ opened: true, message: '{"kind":"Hello"}' });
});

test('renews an expired sign-in to keep live updates going', async ({ page, context }) => {
	await context.addCookies([
		{ name: 'access_token', value: 'expired', url: 'http://localhost:4175' },
		{ name: 'refresh_token', value: 'e2e-refresh', url: 'http://localhost:4175' },
	]);
	await page.goto('/');
	expect(await openActivityFeed(page)).toEqual({ opened: true, message: '{"kind":"Hello"}' });
});
