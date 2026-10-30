import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ConversationSection from './ConversationSection.svelte';

describe('Conversation & build section', () => {
	it('shows messages, agent activity, lists and empty states from the design sheet', async () => {
		await render(ConversationSection);
		for (const text of [
			'hey',
			'Provider error',
			'Wrote 3 files',
			'Writing src/routes/menu/+page.svelte',
			'deploy #14 · main@a3f9c21 · built in 41s',
			'23 files · main@a3f9c21',
			'salah-daily-tracker-0ab8f8.trian.space',
			"Summarize this week's spending",
			'Nothing scheduled today. Ask Kala, or add one yourself.',
			'loading 4 projects…',
		]) {
			expect(page.getByText(text, { exact: true }).elements(), text).toHaveLength(1);
		}
	});

	it('shows the standalone File Row specimen in the agent activity board, 400 wide', async () => {
		await page.viewport(1512, 900);
		const { container } = await render(ConversationSection);
		// Once inside the Activity Block, once on its own.
		expect(page.getByText('src/routes/+page.svelte', { exact: true }).elements()).toHaveLength(2);
		const row = container.querySelector<HTMLElement>('[data-file-row]')!;
		expect(row.textContent).toContain('src/routes/+page.svelte');
		expect(row.textContent).toContain('+214');
		expect(row.getBoundingClientRect().width).toBe(400);
	});
});
