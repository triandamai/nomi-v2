import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { GitBranch, Lightbulb } from '@lucide/svelte';
import Chip from './Chip.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chips pick something (a model, an option, who to ask) or name what a project has.

describe('Chip', () => {
	it('picks a model, with the model id in mono and a menu behind it', async () => {
		const onclick = vi.fn();
		await render(Chip, { kind: 'model', label: 'deepseek-flash', onclick });
		const chip = page.getByRole('button', { name: 'deepseek-flash' });
		await expect.element(chip).toHaveAttribute('aria-haspopup', 'menu');
		const style = getComputedStyle(chip.element());
		expect(style.height).toBe('30px');
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(getComputedStyle(page.getByText('deepseek-flash').element()).fontFamily).toContain('Geist Mono');
		await chip.click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('picks an option, with its icon', async () => {
		const { container } = await render(Chip, { kind: 'option', label: 'Default', icon: Lightbulb });
		expect(container.querySelector('svg.lucide-lightbulb')?.getAttribute('width')).toBe('14');
		expect(getComputedStyle(page.getByText('Default').element()).fontSize).toBe('12px');
	});

	it('picks who to ask, with their mascot', async () => {
		const { container } = await render(Chip, { kind: 'agent', label: 'Nomi', agent: 'nomi' });
		expect(container.querySelector('svg[viewBox="0 0 96 96"]')?.getAttribute('width')).toBe('20');
		expect(getComputedStyle(page.getByText('Nomi').element()).fontSize).toBe('13px');
	});

	it('names what a project was given, without pretending to be a button', async () => {
		await render(Chip, { kind: 'provision', label: 'git repo', icon: GitBranch });
		expect(page.getByRole('button').elements()).toHaveLength(0);
		const label = page.getByText('git repo').element();
		expect(getComputedStyle(label).fontSize).toBe('11px');
		expect(getComputedStyle(label.parentElement!).height).toBe('24px');
		expect(getComputedStyle(label).color).toBe(tokenColor('nm-text-2'));
	});
});
