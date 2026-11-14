import { m } from '$lib/paraglide/messages';
import { ago } from '$lib/adapters/time';
import type { ProjectSummary } from '$lib/adapters/api';
import type { CrewId } from '$lib/ds/agents/crew';
import type { ProjectStatus } from '$lib/ds/projects/ProjectCard.svelte';
import type { ProjectFilter } from '$lib/ds/projects/ProjectToolbar.svelte';
import type { ProjectSort } from '$lib/ds/projects/SortMenu.svelte';

// The Projects page (App / Projects JyWON): each project as a card in the state the backend knows.
// It's live once it has an address, building while Koda builds it, and a draft before then (or
// built but never published). The backend doesn't say when Koda is waiting on an answer, so the
// "Needs you" tab stays empty until it does (BACKEND_TODO.md).

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
};

const TONES: CrewId[] = ['koda', 'dana', 'kala', 'rena', 'tara', 'maya', 'raga', 'nomi'];

/** The same tint for a project every time: a small hash of its id. */
function toneOf(id: string): CrewId {
	let hash = 0;
	for (const char of id) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return TONES[hash % TONES.length];
}

function statusOf(project: ProjectSummary): ProjectStatus {
	if (project.deploy_url) return 'live';
	if (project.status === 'building') return 'building';
	return 'draft';
}

export function projectCards(projects: ProjectSummary[], now: Date): ProjectCardData[] {
	return projects.map((project) => {
		const chats = project.chats_count ?? 0;
		const deploys = project.deploys_count ?? 0;
		const footer = [chats === 1 ? m.projects_chats_one() : m.projects_chats({ count: chats }), ...(deploys ? [m.projects_deploy({ number: deploys })] : []), now.getTime() - Date.parse(project.updated_at) < 60_000 ? m.projects_just_now() : m.projects_ago({ when: ago(project.updated_at, now) })];
		return {
			id: project.id,
			sessionId: project.session_id,
			name: project.name,
			description: project.description?.trim() || m.projects_started_by({ name: project.created_by }),
			tone: toneOf(project.id),
			status: statusOf(project),
			...(project.deploy_url && { url: project.deploy_url }),
			footer: footer.join(' · '),
			href: `/projects/${project.id}`,
			updated: Date.parse(project.updated_at),
			created: Date.parse(project.created_at),
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
	const found = cards.filter((card) => IN_TAB[filter](card) && (!wanted || [card.name, card.description, card.url ?? ''].some((text) => text.toLowerCase().includes(wanted))));
	const by: Record<ProjectSort, (a: ProjectCardData, b: ProjectCardData) => number> = {
		updated: (a, b) => b.updated - a.updated,
		created: (a, b) => b.created - a.created,
		name: (a, b) => a.name.localeCompare(b.name),
		status: (a, b) => STATUS_ORDER.indexOf(a.status) - STATUS_ORDER.indexOf(b.status) || b.updated - a.updated,
	};
	return [...found].sort(by[sort]);
}
