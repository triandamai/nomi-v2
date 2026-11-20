import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import LandingHero from './LandingHero.svelte';
import LandingCrew from './LandingCrew.svelte';
import LandingMascot from './LandingMascot.svelte';
import './landing.css';

// The landing page (Figma 170:2766): the hero's composer and chips, the crew grid and the mascots.

// The page's tokens hang off [data-landing], as on /welcome.
beforeEach(() => document.body.setAttribute('data-landing', ''));
afterEach(() => {
	document.body.innerHTML = '';
	document.body.removeAttribute('data-landing');
});

const box = (element: Element) => element.getBoundingClientRect();

const hero = (value = '') => ({
	title: 'Kamu cerita. Kru Nomi yang kerjain.',
	lines: [
		['Kamu', 'cerita.'],
		['Kru', 'Nomi'],
		['yang', 'kerjain.'],
	] as [[string, string], [string, string], [string, string]],
	lead: 'Nomi itu satu chat berisi kru agen AI.',
	label: 'Bilang ke Nomi kamu mau apa',
	value,
	action: '?/start',
	attach: 'Lampirkan file (masuk dulu)',
	voice: 'Rekam voice note (masuk dulu)',
	send: 'Kirim (masuk dulu)',
	tryLabel: 'atau coba:',
	chips: [
		{ agent: 'dana' as const, label: 'Catat uang kos' },
		{ agent: 'koda' as const, label: 'Menu warung + pre-order' },
	],
	toasts: [{ agent: 'dana' as const, name: 'Dana', line: 'Belanja Rp 48.500 sudah dicatat' }],
});

describe('LandingHero', () => {
	it('names the page in one heading and posts the composer to the page’s action', async () => {
		await page.viewport(1440, 900);
		await render(LandingHero, hero('Bikinin halaman booking'));
		await expect.element(page.getByRole('heading', { level: 1, name: 'Kamu cerita. Kru Nomi yang kerjain.' })).toBeVisible();
		const field = page.getByRole('textbox', { name: 'Bilang ke Nomi kamu mau apa' });
		await expect.element(field).toHaveValue('Bikinin halaman booking');
		expect(field.element().getAttribute('name')).toBe('text');
		const form = field.element().closest('form')!;
		expect([form.method, form.getAttribute('action')]).toEqual(['post', '?/start']);
		// The design's 560 composer with its 16 inset and 16 radius.
		const style = getComputedStyle(form);
		expect([Math.round(box(form).width), style.paddingTop, style.borderTopLeftRadius]).toEqual([560, '16px', '16px']);
	});

	it('keeps Send off until something is typed, and a chip puts its words in the field', async () => {
		await page.viewport(1440, 900);
		await render(LandingHero, hero());
		const send = page.getByRole('button', { name: 'Kirim (masuk dulu)' });
		await expect.element(send).toBeDisabled();
		await page.getByRole('button', { name: 'Menu warung + pre-order' }).click();
		await expect.element(page.getByRole('textbox', { name: 'Bilang ke Nomi kamu mau apa' })).toHaveValue('Menu warung + pre-order');
		await expect.element(page.getByRole('textbox', { name: 'Bilang ke Nomi kamu mau apa' })).toHaveFocus();
		await expect.element(send).toBeEnabled();
		await userEvent.clear(page.getByRole('textbox', { name: 'Bilang ke Nomi kamu mau apa' }));
		await expect.element(send).toBeDisabled();
	});

	it('floats the crew’s notes beside the column on a wide screen, and drops them where there’s no room', async () => {
		await page.viewport(1440, 900);
		await render(LandingHero, hero());
		const note = page.getByText('Belanja Rp 48.500 sudah dicatat');
		await expect.element(note).toBeVisible();
		document.body.innerHTML = '';
		await page.viewport(390, 844);
		await render(LandingHero, hero());
		await expect.element(page.getByText('Belanja Rp 48.500 sudah dicatat')).not.toBeVisible();
	});
});

describe('LandingCrew', () => {
	it('lays the crew out four to a row in 320-high cards, each its own tint', async () => {
		await page.viewport(1440, 900);
		const crew = (['nomi', 'sora', 'koda', 'iris', 'dana'] as const).map((agent) => ({ agent, name: agent, role: 'role', body: 'body', example: 'example' }));
		await render(LandingCrew, { id: 'kru', title: 'Satu chat, dua belas teman kerja.', lead: 'lead', crew });
		const cards = [...page.getByRole('listitem').elements()];
		expect(cards.map((card) => Math.round(box(card).height))).toEqual([320, 320, 320, 320, 320]);
		expect(Math.round(box(cards[4]).top - box(cards[0]).top)).toBe(336);
		expect(new Set(cards.slice(0, 4).map((card) => getComputedStyle(card).backgroundColor)).size).toBe(4);
		await expect.element(page.getByRole('heading', { level: 2, name: 'Satu chat, dua belas teman kerja.' })).toBeVisible();
	});
});

describe('LandingMascot', () => {
	it('is decoration unless it has a name, at the size it’s given', async () => {
		await render(LandingMascot, { agent: 'koda', size: 72 });
		const plain = document.querySelector('.mascot')!;
		expect([plain.getAttribute('aria-hidden'), Math.round(box(plain).width), Math.round(box(plain).height)]).toEqual(['true', 72, 72]);
		document.body.innerHTML = '';
		await render(LandingMascot, { agent: 'koda', size: 44, label: 'Koda' });
		await expect.element(page.getByRole('img', { name: 'Koda' })).toBeVisible();
	});
});
