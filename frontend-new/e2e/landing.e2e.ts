import { expect, test } from '@playwright/test';

// The landing page (/welcome, Figma 170:2766) from the production server: what a visitor sees, and
// the hero's composer carrying what they typed through sign in to a new chat.

test('a visitor gets the landing page in their language, without errors', async ({ page, context }) => {
	await context.addCookies([{ name: 'PARAGLIDE_LOCALE', value: 'id', url: 'http://localhost:4175' }]);
	const errors: string[] = [];
	page.on('pageerror', (error) => errors.push(error.message));
	await page.goto('/welcome');
	await expect(page).toHaveTitle('Nomi: kamu cerita, kru yang kerjain');
	await expect(page.getByRole('heading', { level: 1, name: 'Kamu cerita. Kru Nomi yang kerjain.' })).toBeVisible();
	for (const name of ['Semudah kirim chat ke teman.', 'Satu chat, dua belas teman kerja.', 'Mau pindah? Bawa semuanya.']) {
		await expect(page.getByRole('heading', { level: 2, name })).toBeVisible();
	}
	await expect(page.getByRole('link', { name: 'Masuk' })).toHaveAttribute('href', '/sign-in');
	await expect(page.getByRole('link', { name: 'Coba gratis' })).toHaveAttribute('href', '/register');
	// The nav's sections are on the page.
	await page.getByRole('navigation', { name: 'Bagian' }).getByRole('link', { name: 'Kenalan sama kru' }).click();
	await expect(page).toHaveURL(/#kru$/);
	expect(errors).toEqual([]);
});

test('what a visitor types goes to sign in, and on to a new chat with it in the field', async ({ page }) => {
	await page.goto('/welcome');
	await page.getByRole('button', { name: 'Prayer time reminders' }).click();
	await expect(page.getByRole('textbox', { name: 'Tell Nomi what you want' })).toHaveValue('Prayer time reminders');
	await page.getByRole('button', { name: 'Send (sign in first)' }).click();
	await expect(page).toHaveURL(`/sign-in?redirect_to=${encodeURIComponent(`/chat/new?q=${encodeURIComponent('Prayer time reminders')}`)}`);
});

test('someone signed in goes to Home', async ({ page, context }) => {
	await context.addCookies([{ name: 'access_token', value: 'e2e-token', url: 'http://localhost:4175' }]);
	await page.goto('/welcome');
	await expect(page).toHaveURL('http://localhost:4175/');
});
