import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { tokenColor } from '../test-utils';
import CodeInput from './CodeInput.svelte';

// Auth / Enter code → Code: six boxes, three and three with a dash between. Typing moves on,
// backspace moves back, arrows move, and a pasted code fills every box.

const digits = () => page.getByRole('group', { name: '6-digit code' }).getByRole('textbox');
const digit = (n: number) => page.getByRole('textbox', { name: `Digit ${n} of 6` });
const submitted = () => (document.querySelector('input[name="code"]') as HTMLInputElement).value;

describe('CodeInput', () => {
	it('draws six 56px boxes in one 380 row, a short dash after the third', async () => {
		await render(CodeInput, { name: 'code' });
		const boxes = digits().elements();
		expect(boxes).toHaveLength(6);
		for (const input of boxes) expect(input.getBoundingClientRect().height).toBe(56);
		const row = boxes[0].closest('[role="group"]')!;
		const dash = row.querySelector('[data-part="separator"]')!;
		expect(dash.getBoundingClientRect()).toMatchObject({ width: 8, height: 2 });
		expect(getComputedStyle(boxes[0]).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(boxes[0]).fontSize).toBe('28px');
	});

	it('moves on with each digit, and hands over the whole code once it is full', async () => {
		const oncomplete = vi.fn();
		await render(CodeInput, { name: 'code', oncomplete });
		await userEvent.click(digit(1));
		await userEvent.keyboard('48');
		await expect.element(digit(3)).toHaveFocus();
		expect(submitted()).toBe('48');
		await userEvent.keyboard('x');
		await expect.element(digit(3)).toHaveValue('');
		await userEvent.keyboard('1597');
		expect(submitted()).toBe('481597');
		expect(oncomplete).toHaveBeenCalledWith('481597');
	});

	it('hands the code over only once the form would submit all six digits', async () => {
		let inForm = '';
		await render(CodeInput, { name: 'code', oncomplete: () => (inForm = submitted()) });
		await userEvent.click(digit(1));
		await userEvent.keyboard('123456');
		await expect.poll(() => inForm).toBe('123456');
	});

	it('fills every box from a pasted code, whatever its spacing', async () => {
		const oncomplete = vi.fn();
		await render(CodeInput, { name: 'code', oncomplete });
		await userEvent.click(digit(3));
		const data = new DataTransfer();
		data.setData('text/plain', ' 042-917 ');
		digit(3).element().dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
		await expect.element(digit(6)).toHaveValue('7');
		expect(submitted()).toBe('042917');
		expect(oncomplete).toHaveBeenCalledWith('042917');
	});

	it('takes a code the browser fills in all at once (one-time-code autofill)', async () => {
		const oncomplete = vi.fn();
		await render(CodeInput, { name: 'code', oncomplete });
		await expect.element(digit(1)).toHaveAttribute('autocomplete', 'one-time-code');
		await digit(1).fill('123456');
		expect(submitted()).toBe('123456');
		expect(oncomplete).toHaveBeenCalledWith('123456');
	});

	it('goes back with backspace and moves with the arrow keys', async () => {
		await render(CodeInput, { name: 'code', value: '481' });
		await userEvent.click(digit(4));
		await userEvent.keyboard('{Backspace}');
		await expect.element(digit(3)).toHaveFocus();
		expect(submitted()).toBe('48');
		await userEvent.keyboard('{ArrowLeft}');
		await expect.element(digit(2)).toHaveFocus();
		await userEvent.keyboard('{ArrowRight}{ArrowRight}');
		await expect.element(digit(4)).toHaveFocus();
		await userEvent.keyboard('{Home}');
		await expect.element(digit(1)).toHaveFocus();
	});

	it('edges filled boxes in border-strong, empty ones in border, and the one being typed in accent', async () => {
		await render(CodeInput, { name: 'code', value: '481' });
		await userEvent.click(digit(4));
		expect(getComputedStyle(digit(1).element()).borderTopColor).toBe(tokenColor('nm-border-strong'));
		expect(getComputedStyle(digit(4).element()).borderTopColor).toBe(tokenColor('nm-accent'));
		expect(getComputedStyle(digit(5).element()).borderTopColor).toBe(tokenColor('nm-border'));
		const caret = digit(4).element().parentElement!.querySelector('[data-part="caret"]')!;
		expect(caret.getBoundingClientRect()).toMatchObject({ width: 2, height: 24 });
	});

	it('empties and goes back to the first box when the page clears a wrong code', async () => {
		const { rerender } = await render(CodeInput, { name: 'code', value: '' });
		await userEvent.click(digit(1));
		await userEvent.keyboard('123456');
		await expect.element(digit(6)).toHaveFocus();
		await rerender({ name: 'code', value: '' });
		for (const input of digits().elements()) expect((input as HTMLInputElement).value).toBe('');
		await expect.element(digit(1)).toHaveFocus();
	});

	it('marks every box when the code was wrong', async () => {
		await render(CodeInput, { name: 'code', value: '123456', invalid: true });
		for (const input of digits().elements()) {
			expect(input.getAttribute('aria-invalid')).toBe('true');
			expect(getComputedStyle(input).borderTopColor).toBe(tokenColor('nm-danger'));
		}
	});
});
