import { m } from '$lib/paraglide/messages';

/**
 * A chat's last message for a list. A message that's entirely one fenced code block (the common
 * "show me some code" reply) makes a useless preview, so it says what kind of code it is instead.
 */
export function previewText(content: string | undefined | null): string {
	if (!content) return m.session_no_messages();
	const fenceMatch = content.trim().match(/^```(\S*)\r?\n[\s\S]*?```$/);
	if (fenceMatch) {
		const lang = fenceMatch[1]?.trim();
		return lang ? m.session_lang_code({ lang }) : m.session_code();
	}
	return content;
}
