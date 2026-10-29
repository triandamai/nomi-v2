import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import BuildSharePanel from './BuildSharePanel.svelte';
import ExportPanel from './ExportPanel.svelte';
import ExportDialog from './ExportDialog.svelte';
import { tokenColor } from '../test-utils';

// Build → Share & export: two dialogs reached from the live card. Export is never behind a plan:
// it takes the same number of clicks as upgrading, and works on Free.

afterEach(() => {
	vi.restoreAllMocks();
});

const url = 'https://warung-kopi-menu-3f9a12.trian.space';

describe('BuildSharePanel', () => {
	it('copies the live link, previews it, and sends it to an app', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		await render(BuildSharePanel, { url, preview: { title: 'Warung Kopi Bu Siti', description: 'Menu and pre-order for pickup' }, onclose: () => {}, ondone: () => {} });
		await page.getByRole('button', { name: 'Copy link' }).click();
		expect(write).toHaveBeenCalledWith(url);
		await expect.element(page.getByText('Warung Kopi Bu Siti')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'WhatsApp' })).toHaveAttribute('href', `https://wa.me/?text=${encodeURIComponent(url)}`);
		await expect.element(page.getByRole('link', { name: 'Email' })).toHaveAttribute('href', `mailto:?body=${encodeURIComponent(url)}`);
	});

	it('chooses who can open it, invites people to build together, and is done', async () => {
		const onvisibility = vi.fn();
		const oninvite = vi.fn();
		const ondone = vi.fn();
		await render(BuildSharePanel, { url, preview: { title: 'Warung' }, onvisibility, oninvite, ondone, onclose: () => {} });
		await expect.element(page.getByRole('radio', { name: 'Public' })).toBeChecked();
		await page.getByRole('radio', { name: 'Unlisted' }).click();
		expect(onvisibility).toHaveBeenCalledWith('unlisted');
		await page.getByRole('button', { name: 'Invite people' }).click();
		expect(oninvite).toHaveBeenCalledOnce();
		const done = page.getByRole('button', { name: 'Done' });
		expect(getComputedStyle(done.element()).backgroundColor).toBe(tokenColor('nm-text'));
		await done.click();
		expect(ondone).toHaveBeenCalledOnce();
	});
});

describe('Export', () => {
	const props = {
		project: 'warung-kopi-menu',
		repo: '9 files · git history · main@4be1c07',
		database: 'warung-kopi-menu.sql · menu_items, orders',
		env: 'DATABASE_URL · PUBLIC_SITE_URL · ADMIN_PIN',
	};

	it('lists everything included, says values stay behind, and offers two equal ways out', async () => {
		const onpush = vi.fn();
		const { container } = await render(ExportPanel, { ...props, onpush, ondownload: () => {}, onclose: () => {} });
		await expect.element(page.getByRole('heading', { name: 'Export warung-kopi-menu' })).toBeVisible();
		await expect.element(page.getByText('Everything Koda built is yours. Included on Free, no fee.')).toBeVisible();
		expect(container.querySelectorAll('svg.lucide-check')).toHaveLength(3);
		expect(getComputedStyle(page.getByText('main@4be1c07', { exact: false }).element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByText('Env var values are not exported. Set them again wherever you deploy.')).toBeVisible();
		const push = page.getByRole('button', { name: 'Push to GitHub' });
		const download = page.getByRole('button', { name: 'Download zip' });
		expect(Math.round(push.element().getBoundingClientRect().width)).toBe(Math.round(download.element().getBoundingClientRect().width));
		expect(push.element().querySelector('svg.lucide-github')).not.toBeNull();
		await push.click();
		expect(onpush).toHaveBeenCalledOnce();
	});

	it('opens as a modal named by its title and closes', async () => {
		const onclose = vi.fn();
		await render(ExportDialog, { open: true, ...props, onpush: () => {}, ondownload: () => {}, onclose });
		await expect.element(page.getByRole('dialog', { name: 'Export warung-kopi-menu' })).toBeVisible();
		await page.getByRole('button', { name: 'Close' }).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		expect(onclose).toHaveBeenCalledOnce();
	});
});
