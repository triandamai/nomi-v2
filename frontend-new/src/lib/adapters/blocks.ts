import { m } from '$lib/paraglide/messages';
import type { CrewId } from '$lib/ds/agents/crew';
import type { ClarifyQuestion } from '$lib/ds/chat/clarify';
import type { Cell } from '$lib/ds/chat/table';
import type { RenderedMessage } from './api';
import { crewIdOf } from './chat';

// What a reply carries beyond its text (its content blocks, the pages it read, the memories it
// used or saved), turned into the design system's cards: one item per thing to draw, in the
// order the reply gives them. The thread itself handles thinking, failures, the quota notice and
// follow-ups, so those are left out here. A view the design system doesn't draw yet comes back as
// `unsupported` with its name, rather than not at all. Pure: the chat page renders and acts.

type Scalar = string | number | boolean;
type Format = 'text' | 'number' | 'money' | 'percent' | 'date' | 'date_time';

export type ReplyBlock =
	| { kind: 'clarify'; key: string; agent: CrewId; title: string; questions: ClarifyQuestion[]; asked: string[] }
	| { kind: 'approval'; key: string; id: string; agent: CrewId; title: string; tool: string; fields: { key: string; value: string }[]; status: 'pending' | 'approved' | 'rejected' | 'cancelled' }
	| { kind: 'plan'; key: string; planId: string; agentSessionId: string; title: string }
	| { kind: 'table'; key: string; title: string; columns: string[]; rows: Cell[][]; source?: string }
	| { kind: 'decision'; key: string; question: string; options: { title: string; detail?: string; suggested?: boolean }[] }
	| { kind: 'form'; key: string; agent: CrewId; title: string; intro?: string; fields: FormField[]; submitLabel?: string }
	| { kind: 'selection'; key: string; agent: CrewId; question: string; items: { name: string; detail?: string; amount?: string }[]; selected: boolean[]; confirmLabel?: string; per?: string }
	| {
			kind: 'cart';
			key: string;
			title: string;
			store: string;
			items: { name: string; detail?: string; quantity: number; price: string; available?: boolean; swap?: string }[];
			subtotal: string;
			delivery?: string;
			total: string;
			note?: string;
			checkoutHref: string;
	  }
	| { kind: 'sources'; key: string; sources: { title: string; detail?: string; kind: 'file' | 'web'; href?: string }[] }
	| { kind: 'link'; key: string; href: string; host: string; title: string; description?: string; image?: string }
	| { kind: 'code'; key: string; code: string; language?: string; file?: string }
	| { kind: 'boundary'; key: string; title: string; reason: string; alternatives: { agent: CrewId; title: string; detail?: string; href?: string; message?: string }[] }
	| { kind: 'replies'; key: string; replies: string[] }
	| { kind: 'reminder'; key: string; id: string; title: string; dueAt: string; recurrence: string | null }
	| { kind: 'connect'; key: string; agent: CrewId; service: string; reason: string }
	| { kind: 'stopped'; key: string; tasks: { agent: CrewId; title: string; detail?: string; state: 'done' | 'stopped' | 'waiting' }[] }
	| { kind: 'voice'; key: string; url: string; text: string }
	| { kind: 'memory_used'; key: string; facts: string[] }
	| { kind: 'memory_saved'; key: string; facts: string[] }
	| { kind: 'unsupported'; key: string; view: string };

export type FormField = { key: string; label: string; type: 'text' | 'textarea' | 'number' | 'date' | 'time' | 'select' | 'multiselect' | 'checkbox'; required: boolean; options: string[]; placeholder?: string; initial?: Scalar };

type Block = Record<string, unknown> & { kind: string };
const str = (value: unknown): string | undefined => (typeof value === 'string' && value.trim() ? value : undefined);
const list = <T = Record<string, unknown>>(value: unknown): T[] => (Array.isArray(value) ? (value as T[]) : []);

/** A value as the person reads it: numbers grouped in their language, money without a symbol. */
export function shown(value: unknown, language: string, format?: Format): string {
	if (value === null || value === undefined) return '—';
	if (typeof value === 'number') {
		if (format === 'percent') return `${new Intl.NumberFormat(language, { maximumFractionDigits: 1 }).format(value)}%`;
		return new Intl.NumberFormat(language, { maximumFractionDigits: 2 }).format(value);
	}
	if (typeof value === 'boolean') return value ? '✓' : '—';
	if ((format === 'date' || format === 'date_time') && typeof value === 'string' && !Number.isNaN(Date.parse(value))) {
		return new Date(value).toLocaleString(language, format === 'date' ? { dateStyle: 'medium' } : { dateStyle: 'medium', timeStyle: 'short' });
	}
	return String(value);
}

const host = (url: string) => {
	try {
		return new URL(url).host.replace(/^www\./, '');
	} catch {
		return url;
	}
};

/** The answers to clarifying questions as one message: each question and what was picked. */
export function answersText(questions: { question: string }[], picked: string[][]): string {
	return questions.flatMap((question, index) => (picked[index]?.length ? [`${question.question} ${picked[index].join(', ')}`] : [])).join('\n');
}

/** A plan's steps, from the checklist in its markdown (`- [ ]` and `- [x]` lines). */
export function checklist(content: string | null | undefined): { text: string; done: boolean }[] {
	return (content ?? '').split('\n').flatMap((line) => {
		const match = line.match(/^\s*[-*]\s+\[([ xX])\]\s+(.+)$/);
		return match ? [{ text: match[2].trim(), done: match[1] !== ' ' }] : [];
	});
}

/** A form's answers as one message: each field's label and what was given. */
export function formText(title: string, fields: FormField[], values: Record<string, string | string[] | boolean>): string {
	const lines = fields.flatMap((field) => {
		const value = values[field.key];
		const text = Array.isArray(value) ? value.join(', ') : typeof value === 'boolean' ? (value ? '✓' : '') : (value ?? '').trim();
		return text ? [`${field.label}: ${text}`] : [];
	});
	return [title, ...lines].filter(Boolean).join('\n');
}

const titleCase = (name: string) => name.replace(/_/g, ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());

function view(block: Block, key: string, agent: CrewId, language: string): ReplyBlock | null {
	switch (block.view) {
		case 'table':
			return {
				kind: 'table',
				key,
				title: str(block.title) ?? '',
				columns: list<{ label: string }>(block.columns).map((column) => column.label),
				rows: list(block.rows).map((row) => list<{ key: string; format?: Format }>(block.columns).map((column) => shown(row[column.key], language, column.format))),
			};
		case 'comparison':
			return {
				kind: 'table',
				key,
				title: str(block.title) ?? str(block.verdict) ?? '',
				columns: ['', ...list<string>(block.options)],
				rows: list<{ label: string; values: Scalar[] }>(block.rows).map((row) => [row.label, ...row.values.map((value) => shown(value, language))]),
			};
		case 'decision':
			return {
				kind: 'decision',
				key,
				question: String(block.question ?? ''),
				options: list<{ label: string; detail?: string; recommended?: boolean }>(block.options).map((option) => ({ title: option.label, detail: option.detail, suggested: option.recommended })),
			};
		case 'form':
			return {
				kind: 'form',
				key,
				agent,
				title: str(block.title) ?? '',
				intro: str(block.intro),
				submitLabel: str(block.submit_label),
				fields: list<Record<string, unknown>>(block.fields).map((field) => ({
					key: String(field.key),
					label: String(field.label ?? field.key),
					type: (field.type as FormField['type']) ?? 'text',
					required: field.required === true,
					options: list<string>(field.options),
					placeholder: str(field.placeholder),
					initial: field.default as Scalar | undefined,
				})),
			};
		case 'selection': {
			const items = list<{ name: string; detail?: string; amount?: number; selected?: boolean }>(block.items);
			return {
				kind: 'selection',
				key,
				agent,
				question: String(block.question ?? ''),
				items: items.map((item) => ({ name: item.name, detail: item.detail, amount: item.amount === undefined ? undefined : shown(item.amount, language) })),
				selected: items.map((item) => item.selected === true),
				confirmLabel: str(block.confirm_label),
				per: str(block.per),
			};
		}
		case 'cart': {
			const items = list<{ name: string; detail?: string; quantity?: number; price?: number; available?: boolean; swap?: string }>(block.items);
			const subtotal = items.reduce((sum, item) => sum + (item.available === false ? 0 : (item.price ?? 0) * (item.quantity ?? 1)), 0);
			const delivery = typeof block.delivery === 'number' ? block.delivery : undefined;
			return {
				kind: 'cart',
				key,
				title: String(block.title ?? ''),
				store: String(block.store ?? ''),
				items: items.map((item) => ({
					name: item.name,
					detail: item.detail,
					quantity: item.quantity ?? 1,
					price: item.price === undefined || item.available === false ? '—' : shown(item.price * (item.quantity ?? 1), language),
					available: item.available,
					swap: item.swap,
				})),
				subtotal: shown(subtotal, language),
				delivery: delivery === undefined ? undefined : shown(delivery, language),
				total: shown(subtotal + (delivery ?? 0), language),
				note: str(block.note),
				checkoutHref: String(block.checkout_url ?? ''),
			};
		}
		case 'sources':
			return {
				kind: 'sources',
				key,
				sources: list<{ title: string; url?: string; detail?: string; kind?: 'web' | 'file' }>(block.items).map((item) => ({
					title: item.title,
					detail: item.detail ?? (item.url ? host(item.url) : undefined),
					kind: item.kind ?? (item.url ? 'web' : 'file'),
					href: item.url,
				})),
			};
		case 'link': {
			const url = String(block.url ?? '');
			return { kind: 'link', key, href: url, host: str(block.site) ?? host(url), title: str(block.title) ?? host(url), description: str(block.description), image: str(block.image) };
		}
		case 'code':
			return { kind: 'code', key, code: String(block.code ?? ''), language: str(block.language), file: str(block.file) };
		case 'boundary':
			return {
				kind: 'boundary',
				key,
				title: String(block.title ?? ''),
				reason: String(block.reason ?? ''),
				alternatives: list<{ title: string; detail?: string; agent?: string; message?: string; url?: string }>(block.alternatives).map((alternative) => ({
					agent: crewIdOf(alternative.agent),
					title: alternative.title,
					detail: alternative.detail,
					href: alternative.url,
					message: alternative.message,
				})),
			};
		case 'quick_replies':
			return { kind: 'replies', key, replies: list<string>(block.replies) };
		case 'suggestions':
			return { kind: 'replies', key, replies: list<string>(block.items) };
		default:
			return { kind: 'unsupported', key, view: String(block.view ?? 'view') };
	}
}

/** What a reply carries beyond its text, in order, ready for the cards that draw it. */
export function replyBlocks(message: RenderedMessage, { language }: { language: string }): ReplyBlock[] {
	const agent = crewIdOf(message.agent_display_name);
	const items: ReplyBlock[] = [];
	(message.content_blocks ?? []).forEach((raw, index) => {
		const block = raw as Block;
		const key = `${message.id}-${index}`;
		switch (block.kind) {
			case 'questions': {
				const questions = list<{ question: string; options: { label: string; detail?: string | null }[]; multiple?: boolean }>(block.questions);
				items.push({
					kind: 'clarify',
					key,
					agent,
					title: str(block.intro) ?? m.clarify_default_title(),
					asked: questions.map((question) => question.question),
					questions: questions.map((question, number) => ({
						topic: m.clarify_topic({ number: String(number + 1) }),
						title: question.question,
						multiple: question.multiple === true,
						options: question.options.map((option) => ({ title: option.label, ...(option.detail && { detail: option.detail }) })),
					})),
				});
				break;
			}
			case 'approval_request':
				items.push({
					kind: 'approval',
					key,
					id: String(block.id),
					agent,
					title: String(block.description ?? block.tool_name ?? ''),
					tool: String(block.tool_name ?? ''),
					fields: Object.entries((block.input as Record<string, unknown>) ?? {}).map(([field, value]) => ({ key: field, value: typeof value === 'string' ? value : JSON.stringify(value) })),
					status: block.status === 'approved' ? 'approved' : block.status === 'denied' ? 'rejected' : block.status === 'cancelled' ? 'cancelled' : 'pending',
				});
				break;
			case 'plan':
				items.push({ kind: 'plan', key, planId: String(block.plan_id), agentSessionId: String(block.agent_session_id), title: String(block.title ?? '') });
				break;
			case 'table':
				items.push({
					kind: 'table',
					key,
					title: str(block.title) ?? '',
					source: str(block.source),
					columns: list<{ label: string }>(block.columns).map((column) => column.label),
					rows: list(block.rows).map((row) => list<{ key: string }>(block.columns).map((column) => shown(row[column.key], language))),
				});
				break;
			case 'view': {
				const mapped = view(block, key, agent, language);
				if (mapped) items.push(mapped);
				break;
			}
			case 'reminder':
				items.push({ kind: 'reminder', key, id: String(block.reminder_id), title: String(block.title ?? ''), dueAt: String(block.due_at ?? ''), recurrence: (block.recurrence as string | null) ?? null });
				break;
			case 'workspace_connect':
				items.push({ kind: 'connect', key, agent: crewIdOf(block.agent_type === 'workspace' ? 'Tara' : message.agent_display_name), service: list<string>(block.services).map(titleCase).join(', ') || 'Google', reason: String(block.reason ?? '') });
				break;
			case 'connector_connect':
				items.push({ kind: 'connect', key, agent, service: titleCase(String(block.connector ?? '')), reason: String(block.reason ?? '') });
				break;
			case 'stopped':
				items.push({
					kind: 'stopped',
					key,
					tasks: list<{ agent: string; title: string; detail?: string | null; state: 'done' | 'stopped' | 'waiting' }>(block.tasks).map((task) => ({
						agent: crewIdOf(task.agent),
						title: task.title,
						...(task.detail && { detail: task.detail }),
						state: task.state,
					})),
				});
				break;
			case 'voice':
				items.push({ kind: 'voice', key, url: String(block.url ?? ''), text: String(block.text ?? '') });
				break;
		}
	});
	const sources = message.sources ?? [];
	if (sources.length) items.push({ kind: 'sources', key: `${message.id}-sources`, sources: sources.map((source) => ({ title: source.title ?? host(source.url), detail: host(source.url), kind: 'web', href: source.url })) });
	if (message.used_memories?.length) items.push({ kind: 'memory_used', key: `${message.id}-used`, facts: message.used_memories.map((memory) => memory.content) });
	if (message.saved_memories?.length && message.sender === 'assistant') items.push({ kind: 'memory_saved', key: `${message.id}-saved`, facts: message.saved_memories.map((memory) => memory.content) });
	return items;
}
