import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Kbd from './Kbd.svelte';
import KeyHint from './KeyHint.svelte';
import { tokenColor } from '../test-utils';

describe('Kbd', () => {
	it('is a keyboard key in 11px mono', async () => {
		await render(Kbd, { keys: '⌘K' });
		const key = page.getByText('⌘K').element();
		expect(key.tagName.toLowerCase()).toBe('kbd');
		const style = getComputedStyle(key);
		expect(style.fontFamily).toContain('Geist Mono');
		expect(style.fontSize).toBe('11px');
		expect(style.height).toBe('20px');
		expect(style.borderTopLeftRadius).toBe('4px');
		expect(style.borderTopColor).toBe(tokenColor('nm-border-strong'));
	});
});

describe('KeyHint', () => {
	it('lists what each key does, quietly', async () => {
		await render(KeyHint, {
			hints: [
				{ keys: '↵', action: 'send' },
				{ keys: '⇧↵', action: 'new line' },
				{ keys: '@', action: 'address crew' },
			],
		});
		for (const text of ['↵ send', '⇧↵ new line', '@ address crew']) {
			const hint = page.getByText(text, { exact: true }).element();
			expect(getComputedStyle(hint).color).toBe(tokenColor('nm-text-3'));
			expect(getComputedStyle(hint).fontFamily).toContain('Geist Mono');
		}
	});
});
