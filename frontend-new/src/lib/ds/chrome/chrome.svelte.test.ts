import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SidebarFixture from './SidebarFixture.svelte';
import SettingsNav from './SettingsNav.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Agents & app chrome: the sidebar every app-shell screen uses, and the settings nav.

describe('Sidebar', () => {
	it('is the app’s navigation: brand, New chat, sections, recents, usage and the person', async () => {
		const oncollapse = vi.fn();
		await render(SidebarFixture, { oncollapse });
		const nav = page.getByRole('navigation', { name: 'Main' });
		const element = nav.element();
		expect(getComputedStyle(element).width).toBe('248px');
		expect(getComputedStyle(element).backgroundColor).toBe(tokenBackground('nm-bg'));
		expect(getComputedStyle(element).borderRightColor).toBe(tokenColor('nm-border'));
		await expect.element(page.getByText('nomi', { exact: true })).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /New chat/ })).toHaveAttribute('href', '/chat/new');
		await expect.element(page.getByRole('link', { name: /Home/ })).toHaveAttribute('aria-current', 'page');
		await expect.element(page.getByRole('meter', { name: 'Free plan' })).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /Plan & usage/ })).toHaveAttribute('href', '/settings/plan');
		await expect.element(page.getByRole('link', { name: 'Settings' })).toHaveAttribute('href', '/settings');
		await page.getByRole('button', { name: 'Collapse sidebar' }).click();
		expect(oncollapse).toHaveBeenCalledOnce();
	});

	it('switches workspace from the Build group', async () => {
		const onswitchworkspace = vi.fn();
		await render(SidebarFixture, { onswitchworkspace });
		const chip = page.getByRole('button', { name: /Trian's space/ });
		expect(getComputedStyle(page.getByText("Trian's space").element()).fontFamily).toContain('Geist Mono');
		await chip.click();
		expect(onswitchworkspace).toHaveBeenCalledOnce();
	});

	it('folds the recents away and back', async () => {
		await render(SidebarFixture);
		const toggle = page.getByRole('button', { name: 'Recents' });
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'true');
		await expect.element(page.getByRole('link', { name: /salah-daily-tracker/ })).toBeVisible();
		await toggle.click();
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'false');
		await expect.element(page.getByRole('link', { name: /salah-daily-tracker/ })).not.toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /All chats/ })).toHaveAttribute('href', '/chats');
	});
});

describe('SettingsNav', () => {
	it('goes back to Nomi, searches settings, lists the groups, and signs out', async () => {
		const onlogout = vi.fn();
		await render(SettingsNav, {
			backHref: '/',
			groups: [
				{ label: 'Account', items: [{ href: '/settings/profile', icon: 'user', label: 'Profile' }, { href: '/settings/plan', icon: 'credit-card', label: 'Plan & usage', count: 'Free' }] },
				{ label: 'AI', items: [{ href: '/settings/crew', icon: 'users', label: 'Crew', count: 10 }] },
			],
			active: '/settings/plan',
			user: { name: 'Mario Prasetyo', detail: 'Free plan · Jakarta' },
			version: 'v0.0.75',
			onlogout,
		});
		await expect.element(page.getByRole('link', { name: /Back to Nomi/ })).toHaveAttribute('href', '/');
		await expect.element(page.getByRole('searchbox', { name: 'Search settings' })).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /Plan & usage/ })).toHaveAttribute('aria-current', 'page');
		expect(getComputedStyle(page.getByText('Account').element()).textTransform).toBe('uppercase');
		expect(getComputedStyle(page.getByText('v0.0.75').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Log out' }).click();
		expect(onlogout).toHaveBeenCalledOnce();
	});
});
