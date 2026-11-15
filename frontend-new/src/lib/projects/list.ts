import { m } from '$lib/paraglide/messages';
import { ago } from '$lib/adapters/time';
import type { ProjectSummary } from '$lib/adapters/api';
import type { CrewId } from '$lib/ds/agents/crew';
import type { ProjectStatus } from '$lib/ds/projects/ProjectCard.svelte';
import type { ProjectFilter } from '$lib/ds/projects/ProjectToolbar.svelte';
import type { ProjectSort } from '$lib/ds/projects/SortMenu.svelte';

// The Projects page (App / Projects JyWON): each project as a card in the state the backend knows.
// Building while Koda is at work in its chat ("changing it" once it's live), deploying while a
// publish started here runs, live once it has an address, stopped when Koda left a build unfinished
// (files written, never reported done), and a draft before then (or built but never published).
// Each card carries its one next action (Flow / Projects — Act from the card syolc): Stop while
// Koda works, Resume once stopped, Deploy for a draft with files where this Nomi can publish; and
// what needs the person: Answer where Koda waits on them, Ask Koda to check a live app that's
// down, Ask Koda to fix a deploy that failed (the live one stays up).

export type ProjectCardData = {
	id: string;
	sessionId: string;
	name: string;
	description: string;
	tone: CrewId;
	status: ProjectStatus;
	url?: string;
	footer: string;
	href: string;
	updated: number;
	created: number;
	/** In place of the status's own word: "Koda is changing it", "Koda is deploying". */
	statusLabel?: string;
	/** At the status line's end: a deploy's time running ("00:24"). */
	meta?: string;
	/** The zone's second line where there's no address to show. */
	detail?: string;
	/** The footer's one action, besides Open on a live card. */
	next: 'stop' | 'resume' | 'deploy' | 'answer' | 'check' | 'fix' | null;
	/** How many questions Koda is waiting on (attention). */
	questions?: number;
	files: number;
	deploys: number;
	/** Its address once it's been published, whatever it's doing now. */
	liveUrl?: string;
};

/** What the list alone doesn't say: whose chats Koda is at work in, publishes started here, whether this Nomi publishes. */
export type CardContext = {
	/** Chats (session ids) a crew member is at work in right now. */
	active?: ReadonlySet<string>;
	/** Projects with a publish running, and since when (ISO). */
	deploying?: Readonly<Record<string, string>>;
	/** Whether this Nomi can publish (it has Shipyard). */
	publishing?: boolean;
};

/** "00:24": how long something has run, minutes and seconds. */
export function runningFor(since: string, now: Date): string {
	const seconds = Math.max(0, Math.round((now.getTime() - Date.parse(since)) / 1000));
	return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
}

const TONES: CrewId[] = ['koda', 'dana', 'kala', 'rena', 'tara', 'maya', 'raga', 'nomi'];

/** The same tint for a project every time: a small hash of its id. */
function toneOf(id: string): CrewId {
	let hash = 0;
	for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return TONES[hash % TONES.length];
}

/** The address as the card shows it and links it (`https://` is added there): the host alone. */
export const hostOf = (url: string) => url.replace(/^https?:\/\//, '').replace(/\/+$/, '');

const filesLabel = (count: number) => (count === 1 ? m.projects_files_one() : m.projects_files({ count }));

type Shown = Pick<ProjectCardData, 'status' | 'statusLabel' | 'meta' | 'detail' | 'next' | 'questions'> & { url?: string };

function stateOf(project: ProjectSummary, context: CardContext, now: Date): Shown {
	const files = project.files_count ?? 0;
	const deploys = project.deploys_count ?? 0;
	const since = context.deploying?.[project.id];
	if (since) return { status: 'building', statusLabel: m.projects_status_deploying(), meta: runningFor(since, now), detail: m.projects_deploying_detail({ number: deploys + 1, files: filesLabel(files) }), next: null };
	if (context.active?.has(project.session_id)) {
		return project.deploy_url
			? { status: 'building', statusLabel: m.projects_status_changing(), detail: m.projects_live_stays({ number: deploys }), next: 'stop' }
			: { status: 'building', next: 'stop' };
	}
	if (project.waiting_on_you) return { status: 'attention', questions: project.waiting_on_you, detail: m.projects_attention_detail(), next: 'answer' };
	if (project.deploy_url && project.live_status === 'down') return { status: 'down', url: hostOf(project.deploy_url), next: 'check' };
	if (project.last_deploy?.status === 'failed') {
		return { status: 'failed', detail: project.deploy_url ? m.projects_failed_live({ number: deploys }) : m.projects_failed_first(), next: 'fix' };
	}
	if (project.deploy_url) return { status: 'live', url: hostOf(project.deploy_url), next: null };
	if (project.status === 'building') return { status: 'stopped', detail: m.projects_stopped_detail(), next: 'resume' };
	return { status: 'draft', detail: files ? m.projects_draft_detail({ files: filesLabel(files) }) : m.projects_draft_empty(), next: files && context.publishing ? 'deploy' : null };
}

export function projectCards(projects: ProjectSummary[], now: Date, context: CardContext = {}): ProjectCardData[] {
	return projects.map((project) => {
		const chats = project.chats_count ?? 0;
		const deploys = project.deploys_count ?? 0;
		const { url, ...shown } = stateOf(project, context, now);
		const footer = [chats === 1 ? m.projects_chats_one() : m.projects_chats({ count: chats }), ...(deploys ? [m.projects_deploy({ number: deploys })] : []), now.getTime() - Date.parse(project.updated_at) < 60_000 ? m.projects_just_now() : m.projects_ago({ when: ago(project.updated_at, now) })];
		return {
			id: project.id,
			sessionId: project.session_id,
			name: project.name,
			description: project.description?.trim() || m.projects_started_by({ name: project.created_by }),
			tone: toneOf(project.id),
			...shown,
			...(url && { url }),
			footer: footer.join(' · '),
			href: `/projects/${project.id}`,
			updated: Date.parse(project.updated_at),
			created: Date.parse(project.created_at),
			files: project.files_count ?? 0,
			deploys,
			...(project.deploy_url && { liveUrl: hostOf(project.deploy_url) }),
		};
	});
}

const IN_TAB: Record<ProjectFilter, (card: ProjectCardData) => boolean> = {
	all: () => true,
	live: (card) => card.status === 'live' || card.status === 'down',
	building: (card) => card.status === 'building' || card.status === 'importing',
	attention: (card) => card.status === 'attention' || card.status === 'failed',
};

export function filterCounts(cards: ProjectCardData[]): Record<ProjectFilter, number> {
	return { all: cards.length, live: cards.filter(IN_TAB.live).length, building: cards.filter(IN_TAB.building).length, attention: cards.filter(IN_TAB.attention).length };
}

const STATUS_ORDER: ProjectStatus[] = ['attention', 'failed', 'down', 'live', 'building', 'importing', 'stopped', 'draft'];

export function visibleCards(cards: ProjectCardData[], filter: ProjectFilter, query: string, sort: ProjectSort): ProjectCardData[] {
	const wanted = query.trim().toLowerCase();
	const found = cards.filter((card) => IN_TAB[filter](card) && (!wanted || [card.name, card.description, card.liveUrl ?? ''].some((text) => text.toLowerCase().includes(wanted))));
	const by: Record<ProjectSort, (a: ProjectCardData, b: ProjectCardData) => number> = {
		updated: (a, b) => b.updated - a.updated,
		created: (a, b) => b.created - a.created,
		name: (a, b) => a.name.localeCompare(b.name),
		status: (a, b) => STATUS_ORDER.indexOf(a.status) - STATUS_ORDER.indexOf(b.status) || b.updated - a.updated,
	};
	return [...found].sort(by[sort]);
}
