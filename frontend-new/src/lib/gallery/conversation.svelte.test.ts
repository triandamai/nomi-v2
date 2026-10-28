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
});
