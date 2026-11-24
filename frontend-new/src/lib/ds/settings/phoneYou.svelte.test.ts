import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Brain, Users } from '@lucide/svelte';
import PhonePerson from './PhonePerson.svelte';
import PhonePlanCard from './PhonePlanCard.svelte';
import PhoneSettingsGroup from './PhoneSettingsGroup.svelte';
import { tokenBackground } from '../test-utils';

// You on a phone (Flow / Mobile — Home, Inbox & You, step 3 O1SXw): who's signed in, the plan, and
// the settings pages in their groups.

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PhonePerson', () => {
	it('is one link to the profile, with initials in a 52 circle when there is no photo', async () => {
		await page.viewport(390, 844);
		await render(PhonePerson, { name: 'Mario Prasetyo', detail: '@mario · Profile', href: '/settings/profile' });
		const link = page.getByRole('link', { name: /Mario Prasetyo.*@mario · Profile/ });
		await expect.element(link).toHaveAttribute('href', '/settings/profile');
		const initials = page.getByText('MP').element();
		expect([box(initials).width, box(initials).height]).toEqual([52, 52]);
	});
});

describe('PhonePlanCard', () => {
	it('reads the share used as a meter and links to Plan & usage', async () => {
		await page.viewport(390, 844);
		await render(PhonePlanCard, { plan: 'Free plan', used: 0.0218, caption: '21.8K / 1M tokens this month', href: '/settings/plan', link: 'Plan & usage' });
		const meter = page.getByRole('meter', { name: '21.8K / 1M tokens this month' });
		await expect.element(meter).toHaveAttribute('aria-valuenow', '2');
		expect(box(meter.element()).height).toBe(6);
		expect(getComputedStyle(meter.element().firstElementChild!).backgroundColor).toBe(tokenBackground('nm-accent'));
		await expect.element(page.getByRole('link', { name: 'Plan & usage' })).toHaveAttribute('href', '/settings/plan');
	});
});

describe('PhoneSettingsGroup', () => {
	it('is a section named by its label, of 52-high links, a value only where given', async () => {
		await page.viewport(390, 844);
		await render(PhoneSettingsGroup, {
			label: 'AI',
			rows: [
				{ href: '/settings/crew', icon: Users, label: 'Crew', meta: '11' },
				{ href: '/settings/memory', icon: Brain, label: 'Memory & context' },
			],
		});
		const section = page.getByRole('region', { name: 'AI' }).element();
		const links = [...section.querySelectorAll('a')];
		expect(links.map((link) => [link.textContent?.replace(/\s+/g, ' ').trim(), Math.round(box(link).height)])).toEqual([
			['Crew 11', 52],
			['Memory & context', 52],
		]);
	});

	it('keeps rows that share a link apart (the docs’ examples all link to #)', async () => {
		await render(PhoneSettingsGroup, {
			label: 'Account',
			rows: [
				{ href: '#', icon: Users, label: 'Profile' },
				{ href: '#', icon: Brain, label: 'General' },
			],
		});
		expect([...page.getByRole('region', { name: 'Account' }).element().querySelectorAll('a')].map((link) => link.textContent?.trim())).toEqual(['Profile', 'General']);
	});
});
