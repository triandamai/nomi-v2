import { expect, test } from '@playwright/test';

// The static gallery (`npm run build:gallery`) works from any path on any static host: the pages
// render with their styles and fonts, hydrate, switch theme and move between the sheets.
// e2e/static-gallery-server.mjs serves it under /shared/nomi-ds/.
const at = (path: string) => `http://127.0.0.1:4181/shared/nomi-ds/${path}`;

test('renders each sheet from a nested path, styled and with the fonts', async ({ page }) => {
	for (const [sheet, heading] of [
		['nomi', 'An instrument, not a toy.'],
		['chat', 'Everything Nomi can put in a reply'],
		['build', 'Everything between describe and a live URL'],
		['projects', 'Every project, its state and its next action'],
	]) {
		await page.goto(at(`design-system/${sheet}.html`));
		await expect(page.getByRole('heading', { level: 1, name: heading })).toBeVisible();
		expect(await page.evaluate(() => document.fonts.ready.then(() => document.fonts.check('13px "Geist Variable"')))).toBe(true);
		expect(await page.evaluate(() => getComputedStyle(document.querySelector('h1')!).fontSize)).toBe(sheet === 'chat' ? '40px' : '28px');
	}
});

test('hydrates: the sheet links, the theme and the controls work', async ({ page }) => {
	await page.goto(at('design-system/nomi.html?theme=light'));
	await expect(page.locator('[data-theme="light"]').first()).toBeAttached();
	// A click before the page has hydrated does nothing (the docs are one bundle, parsed whole at
	// start, which a busy CI runner can take seconds over): click until it takes, then it's on.
	const digest = page.getByRole('switch', { name: 'Daily digest' });
	await expect(async () => {
		if ((await digest.getAttribute('aria-checked')) !== 'true') await digest.click();
		await expect(digest).toHaveAttribute('aria-checked', 'true', { timeout: 1_000 });
	}).toPass({ timeout: 30_000 });
	await page.getByRole('link', { name: 'Chat responses' }).click();
	await expect(page).toHaveURL(/\/shared\/nomi-ds\/design-system\/chat\.html$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Everything Nomi can put in a reply' })).toBeVisible();
});

test('loads nothing from other hosts, and every file of its own is found from a nested path', async ({ page }) => {
	const elsewhere: string[] = [];
	const missing: string[] = [];
	page.on('request', (request) => {
		if (!request.url().startsWith('http://127.0.0.1:4181/') && !request.url().startsWith('data:')) elsewhere.push(request.url());
	});
	// An image imported in code once resolved from the site's root and 404'd under /shared/nomi-ds/.
	page.on('response', (response) => {
		if (response.status() >= 400) missing.push(`${response.status()} ${response.url()}`);
	});
	for (const sheet of ['nomi', 'chat', 'build', 'projects']) {
		await page.goto(at(`design-system/${sheet}.html`), { waitUntil: 'networkidle' });
	}
	expect(elsewhere).toEqual([]);
	expect(missing).toEqual([]);
});
