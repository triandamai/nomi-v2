// A turn that runs again (an edited message, a reply asked for again): the thread drops the
// turn's replies, as the server did, and shows the person's message as it now stands. A reminder
// going off in between isn't a reply, so it stays.
import type { RenderedMessage } from '$lib/types';

export function afterRerun(messages: RenderedMessage[], startId: string, update: Partial<RenderedMessage>): RenderedMessage[] {
	const at = messages.findIndex((message) => message.id === startId);
	if (at < 0) return messages;
	const next = messages.findIndex((message, index) => index > at && message.sender === 'user');
	const end = next < 0 ? messages.length : next;
	const isReminder = (message: RenderedMessage) => message.content_blocks?.some((block) => block.kind === 'reminder') ?? false;
	return [
		...messages.slice(0, at),
		{ ...messages[at], ...update },
		...messages.slice(at + 1, end).filter(isReminder),
		...messages.slice(end),
	];
}
