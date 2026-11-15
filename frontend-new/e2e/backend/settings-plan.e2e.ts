import { readFile } from 'node:fs/promises';
import { expect, test } from '@playwright/test';
import { createAccount, signInAs } from './support/accounts';
import { seedUsage } from './support/seed';

// Settings › Plan & usage against the real backend (App / Settings / Plan & usage C0og49): the
// plan and its allowance, the month's figures, by model and by day, the month before, and the
// usage as a CSV.

test('the plan, this month’s usage and the month before, and the export', async ({ page, context }) => {
	const account = await createAccount();
	await seedUsage(account.email, { model: 'deepseek-flash', input: 3_000, output: 500, costUsd: 0.05 });
	await seedUsage(account.email, { model: 'gemini-flash', input: 1_000, output: 200, costUsd: 0.01 });
	await signInAs(context, account);
	await page.goto('/settings/general', { waitUntil: 'networkidle' });
	await page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: /Plan & usage/ }).click();
	await expect(page).toHaveURL(/\/settings\/plan$/);

	// The plan and its allowance.
	await expect(page.getByRole('region', { name: 'Your plan' })).toBeVisible();
	await expect(page.getByRole('meter', { name: 'Allowance' })).toHaveAttribute('aria-valuetext', /^4\.7K \/ .+ tokens$/);
	await expect(page.getByText(/projected by/)).toBeVisible();

	// The month's figures, by model and by day.
	await expect(page.getByRole('definition').filter({ hasText: '$0.06' })).toBeVisible();
	await expect(page.getByRole('definition').filter({ hasText: 'across 2 models' })).toBeVisible();
	await expect(page.getByText('deepseek-flash', { exact: true })).toBeVisible();
	await expect(page.getByText(/^3\.5K tok · 74% · Nomi’s allowance$/)).toBeVisible();
	await expect(page.getByText('today', { exact: true })).toBeVisible();
	await page.getByRole('radio', { name: 'Spend' }).click();
	await expect(page.getByText(/^peak \$0\.06 · /)).toBeVisible();

	// The month before: nothing used, and no projection.
	const months = page.locator('#usage header').getByRole('link');
	await months.first().click();
	await expect(page).toHaveURL(/\?month=\d{4}-\d{2}$/);
	await expect(page.getByRole('heading', { level: 2, name: /^Usage in / })).toBeVisible();
	await expect(page.getByText('nothing used yet')).toBeVisible();
	await expect(page.getByText(/projected by/)).toBeHidden();

	// Back to this month, then the export.
	await months.last().click();
	await expect(page.getByRole('heading', { level: 2, name: 'Usage this month' })).toBeVisible();
	const download = page.waitForEvent('download');
	await page.getByRole('banner').getByRole('link', { name: 'Export usage CSV' }).click();
	const file = await (await download).path();
	const csv = await readFile(file!, 'utf8');
	expect(csv.split('\n')[0]).toBe('date,tokens,spend_usd');
	expect(csv).toContain('deepseek-flash,nomi,3500,1,0.05');
});

test('a plan without a price offers its waitlist, and joining it stays joined', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/settings/plan', { waitUntil: 'networkidle' });
	// Pro has no price on a fresh Nomi: people join its waitlist (the pen's "Join the waitlist").
	const pro = page.getByRole('article', { name: 'Pro' });
	await expect(pro.getByText('Coming soon')).toBeVisible();
	// No price yet: the price's place says how much more room it gives (the pen's "10×").
	await expect(pro.getByText('10×')).toBeVisible();
	await pro.getByRole('button', { name: 'Join the waitlist' }).click();
	await expect(pro.getByText('You’re on the waitlist')).toBeVisible();
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByRole('article', { name: 'Pro' }).getByText('You’re on the waitlist')).toBeVisible();
	await expect(page.getByRole('article', { name: 'Free' }).getByRole('button')).toHaveCount(0);
});

test('coming back from the payment page with a payment that isn’t theirs shows nothing about it', async ({ page, context }) => {
	const account = await createAccount();
	await signInAs(context, account);
	await page.goto('/billing?payment=00000000-0000-0000-0000-000000000000', { waitUntil: 'networkidle' });
	await expect(page).toHaveURL(/\/settings\/plan\?payment=/);
	await expect(page.getByRole('heading', { name: 'Plan & usage', level: 1 })).toBeVisible();
	await expect(page.getByText(/Payment received|Waiting for your payment|didn’t go through|expired/)).toHaveCount(0);
});
