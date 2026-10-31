import { readdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { expect, test } from '@playwright/test';

// The design-system docs, as the team gets them: the static copy (`npm run build:gallery`) served
// from a nested path by e2e/static-gallery-server.mjs. Pages link to each other, search finds any
// page, props come from the code, the theme choice sticks, and every page loads cleanly.
const at = (path: string) => `http://127.0.0.1:4181/shared/nomi-ds/${path}`;
const root = fileURLToPath(new URL('../build-gallery/', import.meta.url));
const pages = (dir: string): string[] =>
	readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
		entry.isDirectory() ? pages(join(dir, entry.name)) : entry.name.endsWith('.html') ? [relative(root, join(dir, entry.name))] : [],
	);

test('opens on the principles and leads into every part', async ({ page }) => {
	await page.goto(at('design-system.html'));
	await expect(page.getByRole('heading', { level: 1, name: 'An instrument, not a toy.' })).toBeVisible();
	await page.getByRole('navigation', { name: 'Design system' }).getByRole('link', { name: 'Colour' }).click();
	await expect(page).toHaveURL(/\/shared\/nomi-ds\/design-system\/foundations\/colour\.html$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Colour' })).toBeVisible();
	await expect(page.getByRole('link', { name: 'Colour', exact: true })).toHaveAttribute('aria-current', 'page');
});

test('finds a component by name and shows its props as the code declares them', async ({ page }) => {
	await page.goto(at('design-system.html'), { waitUntil: 'networkidle' });
	await page.keyboard.press('ControlOrMeta+k');
	await expect(page.getByRole('searchbox', { name: 'Search the design system' })).toBeFocused();
	await page.keyboard.type('segmented');
	await page.keyboard.press('Enter');
	await expect(page).toHaveURL(/\/design-system\/components\/segmented\.html$/);
	await page.getByRole('tab', { name: 'Code' }).click();
	const props = page.getByRole('table');
	await expect(props.getByRole('rowheader', { name: /^surface/ })).toBeVisible();
	await expect(props.getByRole('row', { name: /surface/ })).toContainText("'filled' | 'bg' | 'surface'");
	await expect(props.getByRole('row', { name: /surface/ })).toContainText("'filled'");
});

test('remembers the theme across pages', async ({ page }) => {
	await page.goto(at('design-system/tokens.html'));
	await page.getByRole('radiogroup', { name: 'Theme' }).getByRole('radio', { name: 'Light' }).click();
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	await page.goto(at('design-system/components.html'));
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
});

// Every page, in four parts that run side by side.
for (const part of [0, 1, 2, 3]) {
	test(`every page loads with a title, without errors, and only from its own files (part ${part + 1} of 4)`, async ({ page }) => {
		test.setTimeout(240_000);
		const problems: string[] = [];
		let current = '';
		page.on('pageerror', (error) => problems.push(`${current}: ${error.message}`));
		page.on('request', (request) => {
			if (!request.url().startsWith('http://127.0.0.1:4181/') && !request.url().startsWith('data:')) problems.push(`${current}: requested ${request.url()}`);
		});
		const all = pages(root).filter((path) => path.startsWith('design-system'));
		expect(all.length).toBeGreaterThan(200);
		for (const path of all.filter((_, index) => index % 4 === part)) {
			current = path;
			const response = await page.goto(at(path), { waitUntil: 'networkidle' });
			if (response?.status() !== 200) problems.push(`${path}: ${response?.status()}`);
			// Checked once it has hydrated, which is when a wrong route would turn it into an error page.
			const title = await page.locator('h1').first().textContent();
			if (!title || /^\d{3}$/.test(title.trim())) problems.push(`${path}: h1 is "${title}"`);
		}
		expect(problems).toEqual([]);
	});
}
