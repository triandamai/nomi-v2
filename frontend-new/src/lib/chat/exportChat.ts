// A chat as a Markdown file (the chat menu's Export): its title, then each message under who
// said it and when. Built in the browser from what the thread already holds; nothing is sent.
// Ported from the old app's exportChat (frontend/src/lib/exportChat.ts); the crew's thinking stays out.
import type { MessageItem } from '$lib/adapters/api';

export function chatMarkdown(title: string, messages: Pick<MessageItem, 'sender' | 'agent_display_name' | 'content' | 'created_at'>[], you: string): string {
	const said = messages
		.filter((message) => message.content.trim() !== '' && !message.content.startsWith('🧠'))
		.map((message) => {
			const who = message.sender === 'user' ? you : (message.agent_display_name ?? 'Nomi');
			return `**${who}** · ${message.created_at.slice(0, 16).replace('T', ' ')}\n\n${message.content.trim()}`;
		});
	return [`# ${title}`, ...said].join('\n\n---\n\n') + '\n';
}

/** The file's name: the title, made safe for a file system. */
export function chatFileName(title: string): string {
	const slug = title
		.toLowerCase()
		.replace(/[^\p{L}\p{N}]+/gu, '-')
		.replace(/^-|-$/g, '');
	return `${slug || 'chat'}.md`;
}
