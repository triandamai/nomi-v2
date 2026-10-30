import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { ArrowUp, Mic, Plus } from '@lucide/svelte';
import IconButton from './IconButton.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// 30×30 hit area, 16px icon. Every icon button says what it does to screen readers.

describe('IconButton', () => {
	it('is named by its label and draws a 16px icon in a 30px square', async () => {
		await render(IconButton, { icon: Plus, label: 'Attach a file' });
		const button = page.getByRole('button', { name: 'Attach a file' }).element();
		const style = getComputedStyle(button);
		expect([style.width, style.height]).toEqual(['30px', '30px']);
		expect(button.querySelector('svg')?.getAttribute('width')).toBe('16');
		expect(button.querySelector('svg')?.getAttribute('aria-hidden')).toBe('true');
		expect(style.borderTopColor).toBe(tokenColor('nm-border-strong'));
	});

	it('has a quiet ghost variant and a mint Send', async () => {
		const ghost = await render(IconButton, { icon: Mic, label: 'Record', variant: 'ghost' });
		expect(getComputedStyle(ghost.container.querySelector('button')!).color).toBe(tokenColor('nm-text-3'));

		const send = await render(IconButton, { icon: ArrowUp, label: 'Send', variant: 'send' });
		const style = getComputedStyle(send.container.querySelector('button')!);
		expect(style.backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(style.color).toBe(tokenColor('nm-accent-ink'));
	});

	it('greys Send out, rather than fading it, when there is nothing to send', async () => {
		const onclick = vi.fn();
		await render(IconButton, { icon: ArrowUp, label: 'Send', variant: 'send', disabled: true, onclick });
		const button = page.getByRole('button', { name: 'Send' }).element() as HTMLButtonElement;
		expect(button.disabled).toBe(true);
		const style = getComputedStyle(button);
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(style.color).toBe(tokenColor('nm-text-3'));
		expect(style.opacity).toBe('1');
		button.click();
		expect(onclick).not.toHaveBeenCalled();
	});
});
