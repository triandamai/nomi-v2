import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import LandingHero from './LandingHero.svelte';
import LandingCrew from './LandingCrew.svelte';
import LandingMascot from './LandingMascot.svelte';
import LandingFaq from './LandingFaq.svelte';
import LandingCta from './LandingCta.svelte';
import LandingFooter from './LandingFooter.svelte';
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

describe('LandingFaq', () => {
	const items = [
		{ question: 'Harus bisa ngoding dulu?', answer: 'Sama sekali nggak.' },
		{ question: 'Nomi pakai AI apa?', answer: 'Model bawaan Nomi.' },
	];
	const ask = { title: 'Tanya langsung ke Nomi', note: 'biasanya dibalas dalam hitungan detik', prompt: 'Nomi, bisa bikinin tracker tabungan?', start: 'Mulai ngobrol' };

	it('opens the first question to start, and one at a time', async () => {
		await page.viewport(1440, 900);
		await render(LandingFaq, { id: 'tanya', title: 'Masih penasaran?', ask, action: '?/start', items });
		const first = page.getByRole('button', { name: 'Harus bisa ngoding dulu?' });
		const second = page.getByRole('button', { name: 'Nomi pakai AI apa?' });
		await expect.element(first).toHaveAttribute('aria-expanded', 'true');
		await expect.element(page.getByText('Sama sekali nggak.')).toBeVisible();
		await expect.element(page.getByText('Model bawaan Nomi.')).not.toBeVisible();
		await second.click();
		await expect.element(second).toHaveAttribute('aria-expanded', 'true');
		await expect.element(first).toHaveAttribute('aria-expanded', 'false');
		await expect.element(page.getByText('Sama sekali nggak.')).not.toBeVisible();
		await second.click();
		await expect.element(second).toHaveAttribute('aria-expanded', 'false');
	});

	it('draws a closed question 62 high and an open one with its edge, as the design does', async () => {
		await page.viewport(1440, 900);
		await render(LandingFaq, { id: 'tanya', title: 'Masih penasaran?', ask, action: '?/start', items });
		const rows = [...page.getByRole('listitem').elements()];
		expect(Math.round(box(rows[1]).height)).toBe(62);
		// The design's 1.5 edge; Chromium draws it on whole device pixels, so 1 at 1x.
		expect([getComputedStyle(rows[0]).outlineStyle, getComputedStyle(rows[1]).outlineStyle]).toEqual(['solid', 'none']);
		expect(parseFloat(getComputedStyle(rows[0]).outlineWidth)).toBeGreaterThanOrEqual(1);
	});

	it('sends its prompt to the page’s action from “Ask Nomi yourself”', async () => {
		await render(LandingFaq, { id: 'tanya', title: 'Masih penasaran?', ask, action: '?/start', items });
		const form = page.getByRole('button', { name: 'Mulai ngobrol' }).element().closest('form')!;
		expect([form.method, form.getAttribute('action'), new FormData(form).get('text')]).toEqual(['post', '?/start', 'Nomi, bisa bikinin tracker tabungan?']);
	});
});

describe('LandingCta', () => {
	it('is an empty composer with a quiet edge, and the crew along its foot as decoration', async () => {
		await page.viewport(1440, 900);
		await render(LandingCta, {
			title: 'Mau dibantu apa hari ini?',
			lead: 'Ketik aja.',
			label: 'Bilang ke Nomi kamu mau apa',
			placeholder: 'Ingetin aku olahraga tiap Senin, Rabu, Jumat…',
			action: '?/start',
			attach: 'Lampirkan file (masuk dulu)',
			voice: 'Rekam voice note (masuk dulu)',
			send: 'Kirim (masuk dulu)',
		});
		await expect.element(page.getByRole('heading', { level: 2, name: 'Mau dibantu apa hari ini?' })).toBeVisible();
		const field = page.getByRole('textbox', { name: 'Bilang ke Nomi kamu mau apa' });
		await expect.element(field).toHaveAttribute('placeholder', 'Ingetin aku olahraga tiap Senin, Rabu, Jumat…');
		const form = field.element().closest('form')!;
		// Empty: border/default (#e4e4e4); typed: border/strong (#1c1c1c).
		expect(getComputedStyle(form).borderTopColor).toBe('rgb(228, 228, 228)');
		await expect.element(page.getByRole('button', { name: 'Kirim (masuk dulu)' })).toBeDisabled();
		await userEvent.type(field, 'Catat uang kos');
		await expect.element(page.getByRole('button', { name: 'Kirim (masuk dulu)' })).toBeEnabled();
		expect(getComputedStyle(form).borderTopColor).toBe('rgb(28, 28, 28)');
		expect(document.querySelector('.parade')?.getAttribute('aria-hidden')).toBe('true');
		expect(document.querySelectorAll('.parade .mascot')).toHaveLength(12);
	});
});

describe('LandingFooter', () => {
	it('links only where there’s a page, and switches the language', async () => {
		const onlanguage = vi.fn();
		await render(LandingFooter, {
			home: 'Beranda Nomi',
			homeHref: '/welcome',
			navLabel: 'Situs',
			tagline: 'Satu chat, satu kru, semua beres.',
			columns: [{ title: 'Produk', links: [{ href: '#cara-kerja', label: 'Cara kerja' }] }],
			legal: '© 2026 Nomi Labs',
			languageLabel: 'Bahasa',
			languages: [
				{ locale: 'id', label: 'Bahasa Indonesia', current: true },
				{ locale: 'en', label: 'English', current: false },
			],
			onlanguage,
		});
		const links = [...page.getByRole('navigation', { name: 'Situs' }).getByRole('link').elements()];
		expect(links.map((link) => link.getAttribute('href'))).toEqual(['#cara-kerja']);
		await expect.element(page.getByRole('button', { name: 'Bahasa Indonesia' })).toHaveAttribute('aria-pressed', 'true');
		await page.getByRole('button', { name: 'English' }).click();
		expect(onlanguage).toHaveBeenCalledWith('en');
	});
});
