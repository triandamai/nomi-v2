import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Dumbbell, Mail } from '@lucide/svelte';
import ConnectionCard from './ConnectionCard.svelte';
import AppCard from './AppCard.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Connections' cards (App / Settings / Connections OBmgS), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const google = {
	icon: Mail,
	name: 'Google Workspace',
	account: 'ko••@gmail.com',
	status: 'on',
	caption: 'Gmail, Docs, Sheets, Drive and Calendar',
	canLabel: 'Can do',
	can: ['Gmail', 'Calendar'],
	usedBy: [{ agent: 'tara' as const, name: 'Tara' }],
	when: '2h ago',
	manage: 'Manage',
};

describe('ConnectionCard', () => {
	it('is the pen’s 384 × 248 card: 18 in, the 40px logo, its chips and the footer under a line', async () => {
		await page.viewport(1512, 862);
		await render(ConnectionCard, google);
		const card = page.getByRole('article', { name: 'Google Workspace' }).element();
		expect([box(card).height, getComputedStyle(card).backgroundColor, getComputedStyle(card).outlineColor]).toEqual([248, tokenBackground('nm-surface'), tokenColor('nm-border-strong')]);
		const logo = card.querySelector('[data-part="logo"]')!;
		expect([box(logo).width, box(logo).height, box(logo).left - box(card).left, box(logo).top - box(card).top]).toEqual([40, 40, 18, 18]);
		const chip = page.getByText('Calendar', { exact: true }).element();
		expect([box(chip).height, getComputedStyle(chip).backgroundColor]).toEqual([22, tokenBackground('nm-surface-2')]);
		const manage = page.getByRole('button', { name: 'Manage' }).element();
		expect([box(manage).height, box(card).bottom - box(manage).bottom]).toEqual([30, 18]);
		await expect.element(page.getByText('on', { exact: true })).toBeVisible();
		await expect.element(page.getByText('Tara', { exact: true })).toBeVisible();
		await expect.element(page.getByText('· 2h ago')).toBeVisible();
	});

	it('says Manage was asked for', async () => {
		const onmanage = vi.fn();
		await render(ConnectionCard, { ...google, onmanage });
		await page.getByRole('button', { name: 'Manage' }).click();
		expect(onmanage).toHaveBeenCalledOnce();
	});

	it('leaves the time out when it hasn’t been used', async () => {
		await render(ConnectionCard, { ...google, icon: Dumbbell, name: 'Hevy', when: undefined });
		expect(page.getByText(/·/).elements()).toHaveLength(0);
	});
});

describe('AppCard', () => {
	const hevy = { icon: Dumbbell, name: 'Hevy', caption: 'Workouts, routines and body measurements', agent: 'raga' as const, forLabel: 'for Raga', access: 'read', add: 'Add Hevy' };

	it('is the pen’s 136px outlined card: 16 in, the 36px logo, Add and who it’s for', async () => {
		await page.viewport(1512, 862);
		await render(AppCard, hevy);
		const card = page.getByRole('article', { name: 'Hevy' }).element();
		expect([box(card).height, getComputedStyle(card).outlineColor]).toEqual([136, tokenColor('nm-border')]);
		const logo = card.querySelector('[data-part="logo"]')!;
		expect([box(logo).width, box(logo).left - box(card).left, box(logo).top - box(card).top]).toEqual([36, 16, 16]);
		const add = page.getByRole('button', { name: 'Add Hevy' }).element();
		expect([box(add).width, box(add).height, box(card).right - box(add).right, getComputedStyle(add).color]).toEqual([30, 30, 16, tokenColor('nm-text-3')]);
		await expect.element(page.getByText('for Raga')).toBeVisible();
		await expect.element(page.getByText('read', { exact: true })).toBeVisible();
	});

	it('adds, or says why it can’t', async () => {
		const onadd = vi.fn();
		await render(AppCard, { ...hevy, onadd });
		await page.getByRole('button', { name: 'Add Hevy' }).click();
		expect(onadd).toHaveBeenCalledOnce();
		document.body.innerHTML = '';
		await render(AppCard, { ...hevy, unavailable: 'not set up' });
		expect(page.getByRole('button').elements()).toHaveLength(0);
		await expect.element(page.getByText('not set up')).toBeVisible();
	});
});
