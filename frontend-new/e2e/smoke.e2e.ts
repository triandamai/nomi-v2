import { expect, test, type Page } from './support/test';

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
	await context.addCookies([
		{ name: 'nm_theme', value: 'dark', url: 'http://localhost:4175' },
		{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' },
	]);
	const response = await page.goto('/');
	expect(response?.headers()['cache-control']).toBe('no-cache');
	const html = await response!.text();
	expect(html).toMatch(/<html lang="en" data-theme="dark">/);
	await expect(page).toHaveTitle('Nomi');
});

test('opens the app shell even when the backend answers nothing it asks for', async ({ page, context }) => {
	await context.addCookies([{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' }]);
	await page.goto('/');
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText(/, You\.$/);
});

test('sends someone signed out to sign in, and back where they were going after', async ({ request }) => {
	const response = await request.get('/?from=email', { maxRedirects: 0 });
	expect(response.status()).toBe(303);
	expect(response.headers().location).toBe('/sign-in?redirect_to=%2F%3Ffrom%3Demail');
});

test('keeps everyone on the maintenance page while Nomi is down, and lets them back after', async ({ page, context }) => {
	// The stub is down for this device only (e2e/stub-api.mjs).
	await context.addCookies([
		{ name: 'device_id', value: '00000000-0000-4000-8000-00000000d0e5', url: 'http://localhost:4175' },
		{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' },
	]);
	await page.goto('/');
	await expect(page).toHaveURL(/\/maintenance$/);
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Nomi is down for maintenance');
	await expect(page.getByText('Moving to a bigger database.')).toBeVisible();
	await expect(page.getByText(/^Back by /)).toBeVisible();

	await context.clearCookies();
	const response = await page.request.get('/maintenance', { maxRedirects: 0 });
	expect(response.status()).toBe(307);
	expect(response.headers().location).toBe('/');
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

test('gives each browser a device id on its first visit, kept from then on', async ({ page, context }) => {
	await page.goto('/');
	const [device] = (await context.cookies()).filter((cookie) => cookie.name === 'device_id');
	expect(device?.value).toMatch(/^[0-9a-f-]{36}$/);
	expect(device?.httpOnly).toBe(true);
	await page.goto('/');
	expect((await context.cookies()).find((cookie) => cookie.name === 'device_id')?.value).toBe(device.value);
});

test('sends the old addresses the backend links to on to their v2 pages', async ({ request }) => {
	const response = await request.get('/login?redirect_to=%2Fchat', { maxRedirects: 0 });
	expect(response.status()).toBe(307);
	expect(response.headers().location).toBe('/sign-in?redirect_to=%2Fchat');
});

test.describe('on a phone', () => {
	test.use({
		viewport: { width: 390, height: 844 },
		userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
		hasTouch: true,
	});
	test('a settings page that couldn’t load says what happened, offers Try again, and keeps the way back to You', async ({ page, context }) => {
		await context.addCookies([{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' }]);
		await page.goto('/settings/memory');
		await expect(page.getByRole('heading', { level: 1, name: 'Memory & context' })).toBeVisible();
		await expect(page.getByRole('alert')).toContainText('Your memory couldn’t load.');
		await expect(page.getByRole('alert')).toContainText('Nothing you saved is lost.');
		await expect(page.getByRole('main').getByRole('link', { name: 'You', exact: true })).toHaveAttribute('href', '/settings');
		await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
		await page.screenshot({ path: process.env.SHOT_DIR ? `${process.env.SHOT_DIR}/error.png` : 'test-results/phone-settings-error.png' });
		await page.getByRole('button', { name: 'Try again' }).click();
		await expect(page.getByRole('alert')).toContainText('Your memory couldn’t load.');
	});
});
