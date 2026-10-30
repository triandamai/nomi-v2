import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { House } from '@lucide/svelte';
import Icon from './Icon.svelte';

// Lucide only, stroke 1.5: 16px in nav and buttons, 14px inline with text, 12px in chips.

describe('Icon', () => {
	it('draws a 16px Lucide icon with a 1.5 stroke by default, hidden from screen readers', async () => {
		const { container } = await render(Icon, { icon: House });
		const svg = container.querySelector('svg')!;
		expect(svg.getAttribute('width')).toBe('16');
		expect(svg.getAttribute('height')).toBe('16');
		expect(svg.getAttribute('stroke-width')).toBe('1.5');
		expect(svg.getAttribute('aria-hidden')).toBe('true');
		// Drawn in the text colour around it, so it follows the theme and its context.
		expect(getComputedStyle(svg).stroke).toBe(getComputedStyle(svg).color);
	});

	it('comes in the design’s three sizes', async () => {
		for (const size of [14, 12] as const) {
			const { container } = await render(Icon, { icon: House, size });
			const svg = container.querySelector('svg')!;
			expect(svg.getAttribute('width')).toBe(String(size));
		}
	});

	it('is announced when it carries meaning on its own', async () => {
		await render(Icon, { icon: House, label: 'Home' });
		await expect.element(page.getByRole('img', { name: 'Home' })).toBeInTheDocument();
	});
});
