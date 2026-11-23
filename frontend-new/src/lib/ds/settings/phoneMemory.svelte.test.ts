import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import PhoneMemoryProfile from './PhoneMemoryProfile.svelte';
import PhoneMemoryCard from './PhoneMemoryCard.svelte';

// Flow / Mobile — You & Settings, steps 16 and 17 (Memory & context, a kind of memory).

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PhoneMemoryProfile', () => {
	it('shows the profile, when it changed, and Edit as a 44 target', async () => {
		const onedit = vi.fn();
		await render(PhoneMemoryProfile, { text: 'Runs a coffee stall.', empty: 'Nothing yet.', updated: 'updated today', edit: 'Edit', editLabel: 'Edit the profile', onedit });
		await expect.element(page.getByText('Runs a coffee stall.')).toBeVisible();
		await expect.element(page.getByText('updated today')).toBeVisible();
		const edit = page.getByRole('button', { name: 'Edit the profile' });
		expect(Math.round(box(edit.element()).height)).toBe(44);
		await edit.click();
		expect(onedit).toHaveBeenCalledOnce();
	});

	it('says what fills it when it’s empty', async () => {
		await render(PhoneMemoryProfile, { text: '  ', empty: 'Nomi writes this.', edit: 'Edit', onedit: () => {} });
		await expect.element(page.getByText('Nomi writes this.')).toBeVisible();
	});
});

describe('PhoneMemoryCard', () => {
	it('is named by the memory, shows its strength and who learned it, and opens its ⋯', async () => {
		const onmore = vi.fn();
		await render(PhoneMemoryCard, { content: 'Rina runs the till', strength: { level: 3, name: 'steady' }, meta: 'learned by Nomi', more: 'More for this memory', onmore });
		const card = page.getByRole('article', { name: 'Rina runs the till' });
		await expect.element(card).toBeVisible();
		await expect.element(page.getByRole('img', { name: 'steady' })).toBeVisible();
		await expect.element(page.getByText('learned by Nomi')).toBeVisible();
		const more = page.getByRole('button', { name: 'More for this memory' });
		expect(Math.round(box(more.element()).width)).toBe(44);
		await more.click();
		expect(onmore).toHaveBeenCalledOnce();
	});

	it('asks when the crew isn’t sure', async () => {
		await render(PhoneMemoryCard, { content: 'Gym at 06:30', strength: { level: 1, name: 'faint' }, meta: 'not used yet', check: 'Is this still right?', more: 'More', onmore: () => {} });
		await expect.element(page.getByText('Is this still right?')).toBeVisible();
	});
});
