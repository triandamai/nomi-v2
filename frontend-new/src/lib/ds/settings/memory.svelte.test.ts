import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import StrengthBars from './StrengthBars.svelte';
import MemoryFixture from './MemoryFixture.svelte';
import MemoryProfile from './MemoryProfile.svelte';
import MemorySettings from './MemorySettings.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Memory & context's pieces (App / Settings / Memory hANTE), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('StrengthBars', () => {
	it('fills as many of five 12 × 4 bars as its level, in the text colour', async () => {
		await render(StrengthBars, { level: 5, name: 'core' });
		const bars = [...document.querySelectorAll('[aria-hidden="true"] > span')];
		expect(bars).toHaveLength(5);
		expect([box(bars[0]).width, box(bars[0]).height, box(bars[1]).left - box(bars[0]).right]).toEqual([12, 4, 2]);
		expect(getComputedStyle(bars[4]).backgroundColor).toBe(tokenBackground('nm-text'));
		await expect.element(page.getByRole('img', { name: 'core' })).toBeVisible();
	});

	it('warns when faint: one bar and its name in the warning colour', async () => {
		await render(StrengthBars, { level: 1, name: 'faint' });
		const bars = [...document.querySelectorAll('[aria-hidden="true"] > span')];
		expect([getComputedStyle(bars[0]).backgroundColor, getComputedStyle(bars[1]).backgroundColor]).toEqual([tokenBackground('nm-warning'), tokenBackground('nm-surface-2')]);
		expect(getComputedStyle(page.getByText('faint').element()).color).toBe(tokenColor('nm-warning'));
	});
});

describe('a memory topic and its memories', () => {
	it('opens a 53px topic row to its memories, the fading one quieter', async () => {
		await page.viewport(1512, 862);
		await render(MemoryFixture);
		const row = page.getByRole('button', { name: /People/ });
		await expect.element(row).toHaveAttribute('aria-expanded', 'true');
		expect(box(row.element().closest("li")!).height - box(row.element().closest("li")!.querySelector("ul")!).height).toBe(53);
		expect(getComputedStyle(page.getByText('People', { exact: true }).element()).color).toBe(tokenColor('nm-text-2'));
	});

	it('asks whether it’s still right, and confirms it', async () => {
		const onconfirm = vi.fn();
		await render(MemoryFixture, { onconfirm });
		await page.getByRole('button', { name: 'Still right' }).click();
		expect(onconfirm).toHaveBeenCalledOnce();
	});

	it('corrects in place: Enter saves, Escape puts it back', async () => {
		const onsave = vi.fn();
		await render(MemoryFixture, { onsave });
		await page.getByRole('button', { name: 'Correct it' }).click();
		const field = page.getByRole('textbox', { name: 'What Nomi should remember' });
		await expect.element(field).toHaveFocus();
		await field.fill('Siti is my sister in Jakarta');
		await userEvent.keyboard('{Enter}');
		expect(onsave).toHaveBeenCalledWith('Siti is my sister in Jakarta');
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByText('Siti is my sister in Bandung', { exact: true }).last()).toBeVisible();
	});

	it('forgets', async () => {
		const onforget = vi.fn();
		await render(MemoryFixture, { onforget });
		await page.getByRole('button', { name: 'Forget it' }).click();
		expect(onforget).toHaveBeenCalledOnce();
	});
});

describe('who learned a topic', () => {
	it('adds the pen’s 96-wide Learned by column: the member’s mascot and name', async () => {
		await page.viewport(1512, 862);
		await render(MemoryFixture, { learnedBy: { agent: 'dana', name: 'Dana' } });
		const cell = document.querySelector<HTMLElement>('[data-learned-by]')!;
		expect(box(cell).width).toBe(96);
		await expect.element(page.getByText('Dana', { exact: true })).toBeVisible();
		expect(getComputedStyle(page.getByText('Dana', { exact: true }).element()).color).toBe(tokenColor('nm-text-2'));
		expect(document.body.textContent).toContain('Learned by');
	});
});

describe('MemoryProfile', () => {
	const labels = { title: 'Profile', section: 'You', edit: 'Edit the profile', clear: 'Clear the profile', save: 'Save', cancel: 'Cancel', field: 'Your profile', empty: 'Nomi writes this as it learns. You can write it yourself.', tooLong: 'Too long for the profile budget.' };

	it('is the pen’s profile row: 32 avatar, the text, when it changed, edit and clear', async () => {
		await page.viewport(1512, 862);
		await render(MemoryProfile, { labels, initials: 'MP', text: 'Mario, Jakarta. Founder at Alif.', updated: 'updated 21 Oct', count: '312 / 1,375 tok', budget: 1375 });
		const row = document.querySelector<HTMLElement>('[data-profile-row]')!;
		const style = getComputedStyle(row);
		expect([style.paddingTop, style.paddingLeft, style.columnGap, style.borderRadius]).toEqual(['12px', '14px', '14px', '10px']);
		expect(box(row.querySelector('[data-avatar]')!).width).toBe(32);
		await expect.element(page.getByText('312 / 1,375 tok')).toBeVisible();
		await expect.element(page.getByText('updated 21 Oct')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Edit the profile' })).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Clear the profile' })).toBeVisible();
	});

	it('edits in place with the size as it’s typed, refusing more than the budget', async () => {
		const onsave = vi.fn();
		await render(MemoryProfile, { labels, initials: 'MP', text: 'Short.', updated: null, count: '2 / 100 tok', budget: 100, onsave });
		await page.getByRole('button', { name: 'Edit the profile' }).click();
		const field = page.getByRole('textbox', { name: 'Your profile' });
		await expect.element(field).toHaveFocus();
		await field.fill('x'.repeat(401));
		await expect.element(page.getByText('Too long for the profile budget.')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Save' })).toBeDisabled();
		await field.fill('Mario, Jakarta.');
		await expect.element(page.getByText('4 / 100 tok')).toBeVisible();
		await page.getByRole('button', { name: 'Save' }).click();
		expect(onsave).toHaveBeenCalledWith('Mario, Jakarta.');
	});

	it('says what an empty profile is, and clears one', async () => {
		const onclear = vi.fn();
		const screen = await render(MemoryProfile, { labels, initials: 'MP', text: '', updated: null, count: '0 / 1,000 tok', budget: 1000, onclear });
		await expect.element(page.getByText(labels.empty)).toBeVisible();
		expect(page.getByRole('button', { name: 'Clear the profile' }).elements()).toHaveLength(0);
		await screen.rerender({ labels, initials: 'MP', text: 'Mario.', updated: null, count: '2 / 1,000 tok', budget: 1000, onclear });
		await page.getByRole('button', { name: 'Clear the profile' }).click();
		expect(onclear).toHaveBeenCalledOnce();
	});
});

describe('MemorySettings', () => {
	const labels = {
		learning: 'Learning',
		budget: 'Budget',
		learn: { title: 'Learn from chats', body: 'Keep lasting facts from your conversations.' },
		profile: { title: 'Keep a profile', body: 'A compact summary of who you are and how you work.' },
		search: { title: 'Search past chats', body: 'Let the crew look up details in old chats when relevant.' },
		memoryBudget: 'Memory',
		profileBudget: 'Profile',
		unit: 'tok',
		used: (used: string, budget: string) => `${used} / ${budget} used · loaded into every reply`,
		outOfRange: (min: string, max: string) => `Between ${min} and ${max}.`,
	};
	const values = { learn_from_chats: true, keep_profile: true, search_past_chats: false, memory_budget: 2200, profile_budget: 1375, memory_used: 1812, profile_used: 312 };

	it('is the pen’s 380 column: Learning’s three switches and Budget’s two fields, each [14,20] over a stroke', async () => {
		await page.viewport(1512, 862);
		await render(MemorySettings, { labels, values, language: 'en' });
		const column = document.querySelector<HTMLElement>('[data-memory-settings]')!;
		expect(box(column).width).toBe(380);
		const group = column.querySelector<HTMLElement>('section')!;
		expect([getComputedStyle(group).paddingTop, getComputedStyle(group).paddingLeft, getComputedStyle(group).rowGap]).toEqual(['14px', '20px', '12px']);
		await expect.element(page.getByRole('switch', { name: 'Search past chats' })).not.toBeChecked();
		await expect.element(page.getByRole('textbox', { name: 'Memory' })).toHaveValue('2,200');
		await expect.element(page.getByText('1,812 / 2,200 used · loaded into every reply')).toBeVisible();
	});

	it('fills each track by what’s used, in the warning colour from 80%', async () => {
		await render(MemorySettings, { labels, values, language: 'en' });
		const [memory, profile] = [...document.querySelectorAll<HTMLElement>('[data-track] > span')];
		expect(Math.round((box(memory).width / box(memory.parentElement!).width) * 100)).toBe(82);
		expect(getComputedStyle(memory).backgroundColor).toBe(tokenBackground('nm-warning'));
		expect(getComputedStyle(profile).backgroundColor).toBe(tokenBackground('nm-text-2'));
	});

	it('says each change as it’s made; a budget only once it’s in range', async () => {
		const onchange = vi.fn();
		await render(MemorySettings, { labels, values, language: 'en', onchange });
		await page.getByRole('switch', { name: 'Search past chats' }).click();
		expect(onchange).toHaveBeenLastCalledWith({ search_past_chats: true });
		const field = page.getByRole('textbox', { name: 'Profile' });
		await field.fill('50');
		await userEvent.keyboard('{Enter}');
		await expect.element(page.getByText('Between 100 and 4,000.')).toBeVisible();
		expect(onchange).toHaveBeenCalledTimes(1);
		await field.fill('2000');
		await userEvent.keyboard('{Enter}');
		expect(onchange).toHaveBeenLastCalledWith({ profile_budget: 2000 });
	});
});
