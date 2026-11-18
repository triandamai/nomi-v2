import { m } from '$lib/paraglide/messages';
import type { FailureKind } from '$lib/chat/failure';
import { failureView, type FailureAction } from './chat';

// The phone's chat flows (Flow / Mobile Chat — Clarify → plan GCvfD, Plan → approve → act G1tK6L,
// Approvals & background xvntx, Errors & recovery AyrHG), from the data the desktop already reads:
// the phone's titles and labels for a failure, what changed between two versions of a plan, who
// a plan's steps belong to, when the month's credits reset, and how an approval reads once it's
// decided. Pure, so the phone chat and its tests agree.

export type PhoneFailureAction = { action: FailureAction | 'switch'; label: string; model?: string };

/** A model a failure can move to: its key, its name, how much it reads (tokens) where known. */
export type ModelPick = { key: string; label: string; context?: number | null };

/** The phone's titles (UX contract S4): where the pen names a kind its own way. */
const PHONE_TITLES: Partial<Record<FailureKind, () => string>> = {
	too_long: m.phone_failure_too_long,
	no_model: m.phone_failure_no_model,
	provider_down: m.phone_failure_provider_down,
};

/** The smallest model that reads more than the current one, as "128K"; none when nothing does. */
export function biggerModel(models: ModelPick[], current: string | null): ModelPick | null {
	const now = models.find((model) => model.key === current)?.context ?? 0;
	const bigger = models.filter((model) => model.key !== current && (model.context ?? 0) > now).sort((a, b) => (a.context ?? 0) - (b.context ?? 0));
	return bigger[0] ?? null;
}

/** "128K" for 128,000 tokens. */
export const contextLabel = (tokens: number) => `${Math.round(tokens / 1000)}`;

/**
 * A failed turn on a phone (S4): its title, and its moves, the main one first. The desktop's moves
 * (failureView), labelled as the phone's flow draws them; Retry only where the backend allows it.
 */
export function phoneFailureView(
	kind: FailureKind,
	options: { retry: boolean; canUseCredits: boolean; models?: ModelPick[]; current?: string | null },
): { title: string; actions: PhoneFailureAction[] } {
	const title = (PHONE_TITLES[kind] ?? (() => failureView(kind, options).title))();
	const retry: PhoneFailureAction[] = options.retry ? [{ action: 'retry', label: m.action_retry() }] : [];
	const choose: PhoneFailureAction = { action: 'models', label: m.action_choose_model() };
	const others = (options.models ?? []).filter((model) => model.key !== options.current);
	switch (kind) {
		case 'no_credits':
			return { title, actions: [options.canUseCredits ? { action: 'credits', label: m.action_switch_credits() } : choose, ...retry] };
		case 'rate_limited':
			return { title, actions: [...retry, { action: 'models', label: m.phone_action_another_model() }] };
		case 'too_long': {
			const bigger = biggerModel(options.models ?? [], options.current ?? null);
			return {
				title,
				actions: [
					{ action: 'new_chat', label: m.action_new_chat() },
					bigger?.context ? { action: 'switch', model: bigger.key, label: m.phone_action_bigger_model({ size: contextLabel(bigger.context) }) } : choose,
				],
			};
		}
		case 'bad_key':
			return { title, actions: [{ action: 'own_key', label: m.phone_action_update_key() }, ...(options.canUseCredits ? [{ action: 'credits' as const, label: m.phone_action_use_credits() }] : [])] };
		case 'no_model':
			return { title, actions: [choose] };
		case 'provider_down':
			return { title, actions: [...retry, others[0] ? { action: 'switch', model: others[0].key, label: m.phone_action_try_model({ model: others[0].label }) } : choose] };
		case 'timed_out':
			return { title, actions: [...retry, { action: 'models', label: m.phone_action_faster_model() }] };
		case 'quota':
			return { title, actions: [{ action: 'upgrade', label: m.action_upgrade() }, { action: 'own_key', label: m.action_own_key() }] };
		default: {
			const LABELS: Record<FailureAction, () => string> = {
				retry: m.action_retry,
				credits: m.action_switch_credits,
				models: m.action_choose_model,
				own_key: m.action_own_key,
				upgrade: m.action_upgrade,
				new_chat: m.action_new_chat,
			};
			return { title, actions: failureView(kind, options).actions.map((action) => ({ action, label: LABELS[action]() })) };
		}
	}
}

/** Which steps of a new plan version weren't in the one before (its text isn't there): marked `+`. */
export function addedSteps(previous: string[] | null, current: string[]): boolean[] {
	if (!previous) return current.map(() => false);
	const before = new Set(previous.map((step) => step.trim().toLowerCase()));
	return current.map((step) => !before.has(step.trim().toLowerCase()));
}

/** Who a plan's steps belong to, each once, in order: "Nomi, Dana, Tara". */
export const ownerList = (names: string[]) => [...new Set(names)].join(', ');

/** The first day of next month, when the month's Nomi credits reset: "1 Dec". */
export function resetsOn(now: Date, language: string, timeZone?: string): string {
	const local = new Date(now.toLocaleString('en-US', { timeZone }));
	const first = new Date(Date.UTC(local.getFullYear(), local.getMonth() + 1, 1, 12));
	// Day first in both languages, as the pen writes it ("1 Nov").
	const month = new Intl.DateTimeFormat(language, { month: 'short', timeZone: 'UTC' }).format(first).replace('.', '');
	return `1 ${month}`;
}

/** "0:24" for a countdown. */
export const countdown = (seconds: number) => `${Math.floor(seconds / 60)}:${String(Math.max(0, seconds) % 60).padStart(2, '0')}`;

/** A tool that sends something out (D5): its Deny reads "Don't send"; the rest "Decline". */
export const sendsSomething = (tool: string) => /send|mail|message|reply/i.test(tool);

/** An email the crew wants to send (the preview names it so). */
export const sendsEmail = (tool: string) => /mail/i.test(tool) && /send|reply/i.test(tool);

/** Who an approval's message goes to, from its input. */
export function recipientOf(fields: { key: string; value: string }[]): string | undefined {
	return fields.find((field) => /^(to|recipient|recipients|email)$/i.test(field.key))?.value;
}

/** The message an approval would send, split out of its input: the address, the subject, the text, an attachment. */
export function approvalPreview(fields: { key: string; value: string }[]) {
	const pick = (pattern: RegExp) => fields.find((field) => pattern.test(field.key))?.value;
	return {
		to: recipientOf(fields),
		subject: pick(/^(subject|title)$/i),
		body: pick(/^(body|text|message|content)$/i),
		attachment: pick(/^(attachment|attachments|file|files)$/i),
	};
}

/** "3m 40s" between two times. */
export function finishedIn(fromIso: string, toIso: string): { minutes: number; seconds: number } {
	const total = Math.max(0, Math.round((Date.parse(toIso) - Date.parse(fromIso)) / 1000));
	return { minutes: Math.floor(total / 60), seconds: total % 60 };
}

/** A form's date that's already gone (the person's own day, in their time zone). */
export function isPastDate(value: string, now: Date, timeZone?: string): boolean {
	if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
	const today = new Intl.DateTimeFormat('en-CA', { year: 'numeric', month: '2-digit', day: '2-digit', timeZone }).format(now);
	return value < today;
}

/** "25 Jan 2026" for a form's `YYYY-MM-DD` (the content rules' date), anything else as it is. */
export function formDate(value: string, language: string): string {
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
	if (!match) return value;
	const at = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3]), 12));
	const part = (type: string) => new Intl.DateTimeFormat(language, { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' }).formatToParts(at).find((one) => one.type === type)?.value ?? '';
	return `${part('day')} ${part('month').replace('.', '')} ${part('year')}`;
}

/** A form that is a reminder (a date or a time, and how it repeats), drawn as the pen's reminder card. */
export function isReminderForm(fields: { key: string; label: string; type: string }[]): boolean {
	return fields.some((field) => field.type === 'date' || field.type === 'time') && fields.some((field) => /repeat|recurr/i.test(`${field.key} ${field.label}`));
}

/** An email's fields in the order the person approves them: who it goes to, then the subject, then the rest. */
export function previewOrder<T extends { key: string }>(fields: T[]): T[] {
	const rank = (key: string) => (/^(to|recipient|recipients|email)$/i.test(key) ? 0 : /^(subject|title)$/i.test(key) ? 1 : 2);
	return [...fields].sort((a, b) => rank(a.key) - rank(b.key));
}
