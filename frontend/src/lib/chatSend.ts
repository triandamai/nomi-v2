// Lets a card in a message (questions with ready answers) send a reply through the chat's own
// composer, so it shows and streams like anything the person types.
import { getContext, setContext } from 'svelte';

const KEY = Symbol('nomi-chat-send');

export type ChatSend = (text: string) => void;

export function provideChatSend(send: ChatSend): void {
	setContext(KEY, send);
}

export function useChatSend(): ChatSend | null {
	return getContext<ChatSend | undefined>(KEY) ?? null;
}

/** The person's answers as one message: each question with what they picked, then their note. */
export function composeAnswers(questions: { question: string }[], picked: string[][], note: string): string {
	const lines = questions.flatMap((q, i) => (picked[i]?.length ? [`${q.question} ${picked[i].join(', ')}`] : []));
	const extra = note.trim();
	if (extra) lines.push(extra);
	return lines.join('\n');
}
