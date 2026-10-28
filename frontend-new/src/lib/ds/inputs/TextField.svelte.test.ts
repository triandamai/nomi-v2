import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import TextField from './TextField.svelte';
import { tokenColor } from '../test-utils';

// Label · field · helper. Inputs sit on surface with border-strong; focus adds the 3px focus ring;
// errors swap the border and say what to do next.

describe('TextField', () => {
	it('labels its input and describes it with the helper', async () => {
		await render(TextField, { label: 'Project name', helper: 'Lowercase, numbers and dashes. Becomes your URL.', value: 'budget-keluarga' });
		const input = page.getByRole('textbox', { name: 'Project name' });
		await expect.element(input).toHaveValue('budget-keluarga');
		await expect.element(input).toHaveAccessibleDescription('Lowercase, numbers and dashes. Becomes your URL.');
		const box = input.element().parentElement!;
		expect(getComputedStyle(box).height).toBe('34px');
		expect(getComputedStyle(box).borderTopColor).toBe(tokenColor('nm-border-strong'));
	});

	it('writes machine values (slugs, keys, URLs) in mono', async () => {
		await render(TextField, { label: 'Project name', value: 'budget-keluarga', mono: true });
		expect(getComputedStyle(page.getByRole('textbox').element()).fontFamily).toContain('Geist Mono');
	});

	it('takes typing and shows the focus ring', async () => {
		await render(TextField, { label: 'Project name' });
		const input = page.getByRole('textbox', { name: 'Project name' });
		await userEvent.click(input);
		await userEvent.keyboard('warung');
		await expect.element(input).toHaveValue('warung');
		const box = getComputedStyle(input.element().parentElement!);
		expect(box.borderTopColor).toBe(tokenColor('nm-accent'));
		expect(box.boxShadow).toContain('3px');
	});

	it('says what is wrong and what to do, in place of the helper', async () => {
		await render(TextField, {
			label: 'Project name',
			value: 'Budget Keluarga!',
			helper: 'Lowercase, numbers and dashes. Becomes your URL.',
			error: 'Use lowercase and dashes, e.g. budget-keluarga.',
		});
		const input = page.getByRole('textbox', { name: 'Project name' });
		await expect.element(input).toHaveAttribute('aria-invalid', 'true');
		await expect.element(input).toHaveAccessibleDescription('Use lowercase and dashes, e.g. budget-keluarga.');
		expect(getComputedStyle(input.element().parentElement!).borderTopColor).toBe(tokenColor('nm-danger'));
		expect(page.getByText('Lowercase, numbers and dashes. Becomes your URL.').elements()).toHaveLength(0);
	});

	it('can be disabled, with the reason as its helper', async () => {
		await render(TextField, { label: 'Project name', value: 'salah-daily-tracker', disabled: true, helper: "Project name can't change after deploy." });
		await expect.element(page.getByRole('textbox', { name: 'Project name' })).toBeDisabled();
	});
});
