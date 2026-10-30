import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ActionsSection from './ActionsSection.svelte';

describe('Actions section', () => {
	it('shows every variant and state from the design sheet, each captioned with its name', async () => {
		await render(ActionsSection);
		for (const caption of [
			'Primary',
			'Inverse',
			'Secondary',
			'Ghost',
			'Danger',
			'Primary / Hover',
			'Primary / Focus',
			'Secondary / Hover',
			'Secondary / Focus',
			'Secondary / Active',
			'Primary / Disabled',
			'Inverse / Loading',
			'Icon / Outline',
			'Icon / Ghost',
			'Icon / Send',
			'Icon / Send / Disabled',
			'Link',
			'Link / Inline',
			'Kbd',
			'Key hint',
		]) {
			expect(page.getByText(caption, { exact: true }).elements(), caption).toHaveLength(1);
		}
		expect(page.getByRole('button', { name: 'Deploying · 00:14' }).element().getAttribute('aria-busy')).toBe('true');
	});
});
