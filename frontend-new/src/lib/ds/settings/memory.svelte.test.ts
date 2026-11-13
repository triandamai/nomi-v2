import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import StrengthBars from './StrengthBars.svelte';
import MemoryFixture from './MemoryFixture.svelte';
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
