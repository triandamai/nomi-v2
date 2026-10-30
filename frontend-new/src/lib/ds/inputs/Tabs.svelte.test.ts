import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import TabsFixture from './TabsFixture.svelte';
import { tokenColor } from '../test-utils';

describe('Tabs', () => {
	it('marks the open tab with a line under it and shows its panel', async () => {
		await render(TabsFixture);
		const transcript = page.getByRole('tab', { name: 'Transcript' });
		await expect.element(transcript).toHaveAttribute('aria-selected', 'true');
		const style = getComputedStyle(transcript.element());
		expect(style.height).toBe('40px');
		expect(style.borderBottomColor).toBe(tokenColor('nm-text'));
		expect(getComputedStyle(page.getByRole('tab', { name: 'Events' }).element()).color).toBe(tokenColor('nm-text-3'));
		await expect.element(page.getByRole('tabpanel', { name: 'Transcript' })).toHaveTextContent('Transcript panel');
	});

	it('moves between tabs with the arrow keys', async () => {
		await render(TabsFixture);
		await page.getByRole('tab', { name: 'Transcript' }).click();
		await userEvent.keyboard('{ArrowRight}');
		await expect.element(page.getByRole('tab', { name: 'Events' })).toHaveFocus();
		await expect.element(page.getByRole('tabpanel', { name: 'Events' })).toHaveTextContent('Events panel');
	});
});
