import { describe, expect, it } from 'vitest';
import { chatFileName, chatMarkdown } from './exportChat';

describe('chatMarkdown', () => {
	it('writes the title, then each message under who said it and when', () => {
		const markdown = chatMarkdown(
			'October food',
			[
				{ sender: 'user', agent_display_name: null, content: 'How much on food?', created_at: '2026-10-31T09:15:00Z' },
				{ sender: 'assistant', agent_display_name: 'Dana', content: 'Rp 1.250.000.', created_at: '2026-10-31T09:15:04Z' },
				{ sender: 'assistant', agent_display_name: 'Dana', content: '  ', created_at: '2026-10-31T09:15:05Z' },
			],
			'You',
		);
		expect(markdown).toBe('# October food\n\n---\n\n**You** · 2026-10-31 09:15\n\nHow much on food?\n\n---\n\n**Dana** · 2026-10-31 09:15\n\nRp 1.250.000.\n');
	});
});

describe('chatFileName', () => {
	it('makes the title safe for a file', () => {
		expect(chatFileName('Trip: Bali / Lombok!')).toBe('trip-bali-lombok.md');
		expect(chatFileName('???')).toBe('chat.md');
	});
});
