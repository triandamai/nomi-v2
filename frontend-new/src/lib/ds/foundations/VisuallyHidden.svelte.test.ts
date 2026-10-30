import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import VisuallyHidden from './VisuallyHidden.svelte';

// Runs in a real browser with the app's Tailwind CSS: if the CSS pipeline didn't run, the text
// would be visible and this would fail, so it also guards the test setup itself.

describe('VisuallyHidden', () => {
	it('keeps text for screen readers but takes no visible space', async () => {
		await render(VisuallyHidden, { text: 'Unread messages' });

		const element = page.getByText('Unread messages').element() as HTMLElement;
		const style = getComputedStyle(element);
		expect(style.position).toBe('absolute');
		expect(element.getBoundingClientRect().width).toBeLessThanOrEqual(1);
		expect(element.getBoundingClientRect().height).toBeLessThanOrEqual(1);
		expect(style.overflow).toBe('hidden');
	});
});
