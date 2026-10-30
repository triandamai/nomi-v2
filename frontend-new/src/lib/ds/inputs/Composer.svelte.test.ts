import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Composer from './Composer.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// One composer for chat, home and project: the agent chip and placeholder change.

const props = { placeholder: 'Ask Nomi, or hand something to the crew…', agent: { id: 'nomi', name: 'Nomi' }, model: 'deepseek-flash' } as const;

describe('Composer', () => {
	it('is a panel with the field over its tools', async () => {
		await render(Composer, { ...props });
		const field = page.getByRole('textbox', { name: 'Message' });
		await expect.element(field).toHaveAttribute('placeholder', props.placeholder);
		expect(getComputedStyle(field.element()).fontSize).toBe('15px');
		const panel = getComputedStyle(field.element().closest('[data-composer]')!);
		expect(panel.backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(panel.borderTopColor).toBe(tokenColor('nm-border-strong'));
		expect(panel.borderTopLeftRadius).toBe('10px');
		for (const name of ['Attach a file', 'Nomi', 'deepseek-flash', 'Record a voice note']) {
			await expect.element(page.getByRole('button', { name })).toBeInTheDocument();
		}
	});

	it('only offers Send when there is something to send', async () => {
		await render(Composer, { ...props });
		const send = page.getByRole('button', { name: 'Send' });
		await expect.element(send).toBeDisabled();
		await userEvent.type(page.getByRole('textbox', { name: 'Message' }), '   ');
		await expect.element(send).toBeDisabled();
		await userEvent.type(page.getByRole('textbox', { name: 'Message' }), 'hi');
		await expect.element(send).toBeEnabled();
	});

	it('sends with Enter, keeps Shift+Enter for a new line, and clears after sending', async () => {
		const onsend = vi.fn();
		await render(Composer, { ...props, onsend });
		const field = page.getByRole('textbox', { name: 'Message' });
		await userEvent.type(field, 'Food spend in October');
		await userEvent.keyboard('{Shift>}{Enter}{/Shift}');
		await userEvent.keyboard('by week');
		await expect.element(field).toHaveValue('Food spend in October\nby week');
		await userEvent.keyboard('{Enter}');
		expect(onsend).toHaveBeenCalledWith('Food spend in October\nby week');
		await expect.element(field).toHaveValue('');
	});

	it('does not send while an input method is composing a word', async () => {
		const onsend = vi.fn();
		await render(Composer, { ...props, onsend });
		const field = page.getByRole('textbox', { name: 'Message' }).element() as HTMLTextAreaElement;
		field.focus();
		await userEvent.type(field, 'にほん');
		field.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', isComposing: true, bubbles: true }));
		expect(onsend).not.toHaveBeenCalled();
	});

	it('sends with the Send button too, and hands the chips’ clicks out', async () => {
		const onsend = vi.fn();
		const onpickagent = vi.fn();
		const onattach = vi.fn();
		await render(Composer, { ...props, onsend, onpickagent, onattach });
		await userEvent.type(page.getByRole('textbox', { name: 'Message' }), 'hello');
		await page.getByRole('button', { name: 'Send' }).click();
		expect(onsend).toHaveBeenCalledWith('hello');
		await page.getByRole('button', { name: 'Nomi' }).click();
		await page.getByRole('button', { name: 'Attach a file' }).click();
		expect(onpickagent).toHaveBeenCalledOnce();
		expect(onattach).toHaveBeenCalledOnce();
	});
});
