import { expect, test } from '@playwright/test';

// The static gallery (`npm run build:gallery`) works from any path on any static host: the pages
// render with their styles and fonts, hydrate, switch theme and move between the three sheets.
// e2e/static-gallery-server.mjs serves it under /shared/nomi-ds/.
const at = (path: string) => `http://127.0.0.1:4181/shared/nomi-ds/${path}`;

test('renders each sheet from a nested path, styled and with the fonts', async ({ page }) => {
	for (const [sheet, heading] of [
		['nomi', 'An instrument, not a toy.'],
		['chat', 'Everything Nomi can put in a reply'],
		['build', 'Everything between describe and a live URL'],
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
	await page.getByRole('switch', { name: 'Daily digest' }).click();
	await expect(page.getByRole('switch', { name: 'Daily digest' })).toHaveAttribute('aria-checked', 'true');
	await page.getByRole('link', { name: 'Chat responses' }).click();
	await expect(page).toHaveURL(/\/shared\/nomi-ds\/design-system\/chat\.html$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Everything Nomi can put in a reply' })).toBeVisible();
});

test('loads nothing from other hosts, so it works where only its own files are allowed', async ({ page }) => {
	const elsewhere: string[] = [];
	page.on('request', (request) => {
		if (!request.url().startsWith('http://127.0.0.1:4181/') && !request.url().startsWith('data:')) elsewhere.push(request.url());
	});
	for (const sheet of ['nomi', 'chat', 'build']) {
		await page.goto(at(`design-system/${sheet}.html`), { waitUntil: 'networkidle' });
	}
	expect(elsewhere).toEqual([]);
});
