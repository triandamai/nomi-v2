import { m } from '$lib/paraglide/messages';
import { CREW, type CrewId } from '$lib/ds/agents/crew';
import type { MentionOption } from '$lib/ds/chat/mention';
import { clock, dayKey, dayLabel } from '$lib/chat/dates';
import type { FailureKind } from '$lib/chat/failure';
import { pinnedFirst, searchChats } from '$lib/chat/sessionOrder';
import type { HomeSummary, ModelOptions, SessionSummary } from './api';
import { chatHref } from './shell';

// The chat's data, turned from the API's shapes into the design system's props: who a message is
// from, the starters on a blank chat, the mention picker's crew, Home's Today strip, the chats
// list, a failure's title and next moves, and the model picker. Pure, so the server render and
// the page agree.

const BY_NAME = new Map<string, CrewId>(CREW.map((member) => [member.name.toLowerCase(), member.id]));

/** The mascot a message wears, from the name it carries ("Dana"); Nomi's for anyone else. */
export function crewIdOf(name: string | null | undefined): CrewId {
	return (name && BY_NAME.get(name.trim().toLowerCase())) || 'nomi';
}

export type Starter = { agent: CrewId; label: string; route: string; text: string };

/** The blank chat's starters (App / Home → Suggestions Ce004), each routed to who owns it. */
const STARTERS: { owner: string; label: () => string }[] = [
	{ owner: 'Dana', label: m.starter_spending },
	{ owner: 'Rena', label: m.starter_saturday },
	{ owner: 'Koda', label: m.starter_website },
	{ owner: 'Kala', label: m.starter_stretch },
];

/** The starters whose owner is on the crew. Picking one asks it of them (an @mention). */
export function starters(crew: { name: string }[]): Starter[] {
	const names = new Set(crew.map((member) => member.name));
	return STARTERS.filter((starter) => names.has(starter.owner)).map((starter) => {
		const agent = crewIdOf(starter.owner);
		return { agent, label: starter.label(), route: `→ ${agent}`, text: withMention(starter.label(), starter.owner) };
	});
}

/** The crew to "@" in the composer (those the design has a mascot for), by the names the server
 * routes on (nomi_agent_core::mention). */
export function mentionOptions(crew: { agent?: CrewId; name: string; role: string }[]): MentionOption[] {
	return crew.flatMap((member) => (member.agent ? [{ kind: 'crew' as const, agent: member.agent, name: member.name, role: member.role }] : []));
}

/** The crew panel beside a chat: who replied in it says so, and who's answering now says working. */
export function chatCrewRows<T extends { name: string; role: string; state?: string }>(rows: T[], inChat: { replied: string[]; working: string | null }): T[] {
	return rows.map((row) => ({
		...row,
		...(inChat.replied.includes(row.name) && { role: m.crew_replied_here() }),
		...(inChat.working === row.name && { state: m.crew_state_working() }),
	}));
}

/** A message addressed to a crew member (the composer's To chip): "@Dana …", unless it already is. */
export function withMention(text: string, name: string): string {
	if (crewIdOf(name) === 'nomi' || text.trimStart().toLowerCase().startsWith(`@${name.toLowerCase()}`)) return text;
	return `@${name} ${text}`;
}

export type TodayColumn = {
	key: 'out' | 'reminders' | 'plans';
	title: string;
	count: number;
	body: string;
	action: { label: string; href?: string; prompt?: string };
};

/** Home's Today strip (BgqD6): what happened while they were out, today's reminders, plans running. */
export function todayColumns(summary: HomeSummary, timeZone?: string): TodayColumn[] {
	const out = summary.while_you_were_out[0];
	const reminder = summary.today[0];
	const plan = summary.plans[0];
	return [
		{
			key: 'out',
			title: m.today_out_title(),
			count: summary.while_you_were_out_total,
			body: out ? m.today_out_item({ agent: out.agent, title: out.title }) : m.today_out_empty(),
			action: { label: m.today_out_action(), href: '/inbox' },
		},
		{
			key: 'reminders',
			title: m.today_reminders_title(),
			count: summary.today_total,
			body: reminder ? m.today_reminder_item({ label: reminder.label, time: clock(reminder.run_at, timeZone) }) : m.today_reminders_empty(),
			action: { label: m.today_reminders_action(), prompt: withMention(m.today_reminders_prompt(), 'Kala') },
		},
		{
			key: 'plans',
			title: m.today_plans_title(),
			count: summary.plans_total,
			body: plan ? m.today_plan_item({ title: plan.title, done: plan.done, total: plan.total }) : m.today_plans_empty(),
			action: plan ? { label: m.today_plans_open(), href: `/chat/${plan.session_id}` } : { label: m.today_plans_action(), prompt: withMention(m.today_plans_prompt(), 'Rena') },
		},
	];
}

export type ChatListRow = { id: string; href: string; title: string; preview: string; time: string; pinned: boolean; working: boolean };
export type ChatListGroup = { key: string; label: string; rows: ChatListRow[] };

/** What a chat's last message says, in one line (the crew's thinking and file tags left out). */
function preview(text: string | undefined): string {
	return (text ?? '').replace(/^🧠\s*/, '').replace(/\s+/g, ' ').trim();
}

/**
 * The Chats page: the person's chats (not the projects' own), pinned first, then by the day they
 * last moved, each with its last line and time; narrowed to `query` when there's one.
 */
export function chatListGroups(sessions: SessionSummary[], options: { now: Date; timeZone?: string; language: string; query: string }): ChatListGroup[] {
	const chats = searchChats(
		sessions.filter((session) => !session.project_id),
		options.query,
	);
	const row = (session: SessionSummary): ChatListRow => {
		const at = session.last_message?.created_at ?? session.updated_at;
		const today = dayKey(at, options.timeZone) === dayKey(options.now, options.timeZone);
		return {
			id: session.id,
			href: chatHref(session),
			title: session.title?.trim() || m.recents_untitled(),
			preview: preview(session.last_message?.content),
			time: today ? clock(at, options.timeZone) : dayLabel(at, options.language, options.now, options.timeZone),
			pinned: Boolean(session.pinned || session.pinned_at),
			working: Boolean(session.agent_active),
		};
	};
	const ordered = pinnedFirst(chats);
	const pinned = ordered.filter((session) => session.pinned || session.pinned_at);
	const groups: ChatListGroup[] = pinned.length ? [{ key: 'pinned', label: m.chats_pinned(), rows: pinned.map(row) }] : [];
	for (const session of ordered.filter((one) => !(one.pinned || one.pinned_at))) {
		const at = session.last_message?.created_at ?? session.updated_at;
		const key = dayKey(at, options.timeZone);
		let group = groups.find((one) => one.key === key);
		if (!group) groups.push((group = { key, label: dayLabel(at, options.language, options.now, options.timeZone), rows: [] }));
		group.rows.push(row(session));
	}
	return groups;
}

export type FailureAction = 'retry' | 'credits' | 'models' | 'own_key' | 'upgrade' | 'new_chat';

const TITLES: Record<FailureKind, () => string> = {
	no_credits: m.failure_no_credits,
	bad_key: m.failure_bad_key,
	rate_limited: m.failure_rate_limited,
	too_long: m.failure_too_long,
	no_model: m.failure_no_model,
	provider_down: m.failure_provider_down,
	unreachable: m.failure_unreachable,
	unreadable: m.failure_unreadable,
	timed_out: m.failure_timed_out,
	quota: m.failure_quota,
	other: m.failure_other,
};

/**
 * A failed turn's callout: its title, and the moves that fix it, the main one first. Retry only
 * where the backend says trying again can help; switching to Nomi's credits only where the person
 * has Nomi models to switch to.
 */
export function failureView(kind: FailureKind, { retry, canUseCredits }: { retry: boolean; canUseCredits: boolean }): { title: string; actions: FailureAction[] } {
	const again: FailureAction[] = retry ? ['retry'] : [];
	const elsewhere: FailureAction = canUseCredits ? 'credits' : 'models';
	const actions: Record<FailureKind, FailureAction[]> = {
		no_credits: [...again, elsewhere],
		bad_key: ['own_key', ...(canUseCredits ? (['credits'] as const) : [])],
		rate_limited: [...again, 'models'],
		too_long: ['new_chat', 'models'],
		no_model: ['models'],
		provider_down: [...again, 'models'],
		unreachable: again,
		unreadable: [...again, 'models'],
		timed_out: again,
		quota: ['upgrade', 'own_key'],
		other: again,
	};
	return { title: TITLES[kind](), actions: actions[kind] };
}

export type ModelMenuGroup = { group: string; models: { key: string; label: string; source: string; chosen: boolean }[] };

/** The model picker: the models switched on, grouped by who pays (the person's keys first). */
export function modelMenu(options: ModelOptions, chosen: string | null): ModelMenuGroup[] {
	const pick = (source: 'connection' | 'nomi') =>
		options.groups
			.filter((group) => group.source === source)
			.flatMap((group) =>
				group.models
					.filter((model) => model.enabled || model.key === chosen)
					.map((model) => ({ key: model.key, label: model.label, source: source === 'nomi' ? m.models_nomi_credits() : group.label, chosen: model.key === chosen })),
			);
	const keys = pick('connection');
	const credits = pick('nomi');
	return [...(keys.length ? [{ group: m.models_your_keys(), models: keys }] : []), ...(credits.length ? [{ group: m.models_nomi_credits(), models: credits }] : [])];
}
