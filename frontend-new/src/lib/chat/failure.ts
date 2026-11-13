import type { ContentBlock } from '$lib/adapters/api';

// What went wrong with a turn, from what the backend actually reports. A failed turn posts its
// explanation as a message (one plain sentence in the person's language, error.* in
// nomi-i18n/locales) with a `run_failed` block (and a retry when trying again can help), or a
// `quota_notice` card when the month's allowance ran out. It reports no status code, request id
// or provider reason beyond that sentence (BACKEND_TODO.md), so the kind is read from the
// sentence itself: the pattern of each error.* message, in English and Indonesian.

export type FailureKind =
	| 'no_credits'
	| 'bad_key'
	| 'rate_limited'
	| 'too_long'
	| 'no_model'
	| 'provider_down'
	| 'unreachable'
	| 'unreadable'
	| 'timed_out'
	| 'quota'
	| 'other';

/** In the backend's own order (nomi-llm's user_message): credits before a key, before a limit. */
const PATTERNS: [FailureKind, RegExp][] = [
	['quota', /used this month's allowance on Nomi's models|jatah bulan ini untuk model Nomi/i],
	['no_credits', /out of credits for this request|kredit .* habis untuk permintaan ini/i],
	['bad_key', /rejected the API key|menolak API key/i],
	['rate_limited', /limiting how many requests|membatasi jumlah permintaan/i],
	['too_long', /grown too long for the model|terlalu panjang untuk modelnya/i],
	['no_model', /couldn't find the selected model|tidak menemukan model yang dipilih/i],
	['provider_down', /is having trouble right now|sedang mengalami gangguan/i],
	['unreachable', /couldn't reach the AI provider|belum bisa menghubungi penyedia AI/i],
	['unreadable', /sent back a reply I couldn't read|tidak bisa aku baca/i],
	['timed_out', /took too long, so I stopped|kelamaan, jadi aku hentikan/i],
];

export function failureKind(text: string): FailureKind {
	return PATTERNS.find(([, pattern]) => pattern.test(text))?.[0] ?? 'other';
}

export type Failure = { kind: FailureKind; text: string; retry: boolean };

/** A message that's a failure notice, read: its kind, its sentence, and whether it can be retried. */
export function failureOf(message: { content: string; content_blocks: ContentBlock[] | null }): Failure | null {
	const blocks = message.content_blocks ?? [];
	if (blocks.some((block) => block.kind === 'quota_notice')) return { kind: 'quota', text: message.content, retry: false };
	const failed = blocks.find((block) => block.kind === 'run_failed');
	if (!failed) return null;
	return { kind: failureKind(message.content), text: message.content, retry: Boolean((failed as { retry?: unknown }).retry) };
}
