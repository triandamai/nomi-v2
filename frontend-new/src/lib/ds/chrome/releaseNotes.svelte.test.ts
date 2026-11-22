import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import AccountMenu from './AccountMenu.svelte';
import ReleaseNotesDialog from './ReleaseNotesDialog.svelte';

// What's new: the running version's release notes in a Dialog, opened from the account menu.

const releases = [
	{ version: '0.1.7-staging4', date: '22 Nov 2026', url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.7-staging4', html: '<ul><li>Device preview fallback</li></ul>' },
	{ version: '0.1.7-staging3', date: '21 Nov 2026', html: '<ul><li>Action cards under the reply</li></ul>' },
];

describe('ReleaseNotesDialog', () => {
	it('shows the running version’s notes first, then the earlier ones', async () => {
		await render(ReleaseNotesDialog, { open: true, releases });
		const dialog = page.getByRole('dialog', { name: 'What’s new in v0.1.7-staging4' });
		await expect.element(dialog).toBeVisible();
		await expect.element(dialog.getByText('Device preview fallback')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'View on GitHub' })).toHaveAttribute('target', '_blank');
		await expect.element(page.getByRole('region', { name: 'v0.1.7-staging3' })).toHaveTextContent('Action cards under the reply');
		await page.getByRole('button', { name: 'Got it' }).click();
		await expect.element(dialog).not.toBeInTheDocument();
	});

	it('shows nothing without notes', async () => {
		await render(ReleaseNotesDialog, { open: true, releases: [] });
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});

describe('AccountMenu’s What’s new', () => {
	const account = { name: 'Mario Prasetyo', settingsHref: '/settings', profileHref: '/settings/profile', planHref: '/settings/plan', theme: 'dark' as const, language: 'en', languages: [{ value: 'en', label: 'English' }], version: 'v0.1.7', ontheme: vi.fn(), onlanguage: vi.fn(), onlogout: vi.fn() };

	it('opens the release notes when the build has them', async () => {
		const onwhatsnew = vi.fn();
		await render(AccountMenu, { ...account, onwhatsnew });
		await page.getByRole('button', { name: 'Mario Prasetyo' }).click();
		const row = page.getByRole('menuitem', { name: /What’s new/ });
		await expect.element(row).toHaveTextContent('v0.1.7');
		await row.click();
		expect(onwhatsnew).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('menu', { name: 'Account' })).not.toBeInTheDocument();
	});

	it('only shows the version without them', async () => {
		await render(AccountMenu, account);
		await page.getByRole('button', { name: 'Mario Prasetyo' }).click();
		await expect.element(page.getByText('What’s new')).toBeVisible();
		await expect.element(page.getByRole('menuitem', { name: /What’s new/ })).not.toBeInTheDocument();
	});
});
