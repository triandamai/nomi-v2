import { m } from '$lib/paraglide/messages';
import { ago } from '$lib/adapters/time';
import type { OrgRole } from '$lib/adapters/api';
import type { CrewId } from '$lib/ds/agents/crew';
import type { ProjectChat, ProjectParts } from '$lib/ds/projects/ProjectDetailPanel.svelte';
import type { MoveTarget } from '$lib/ds/projects/ProjectMoveDialog.svelte';
import type { MoveChoices, ProjectAbout, ProjectChatRow } from '$lib/server/projects';
import type { ProjectCardData } from './list';

// The Projects page's detail panel (Flow / Projects — Detail panel pgSon) and Move dialog (Card
// menu M9O0iU, step 5), from what the backend says: what's in a project (its files and stack, its
// tables, its deploys, who can open it), its chats, and the workspaces it can move to. A part still
// being read says "checking…"; one that couldn't be read says so, never 0.

const STACKS: Record<string, () => string> = { sveltekit: () => 'SvelteKit', react: () => 'React', vue: () => 'Vue', astro: () => 'Astro', static: m.projects_stack_static };

/** The database a project's files use, by name; null when it uses none. */
export function databaseName(database: string): string | null {
	return database === 'none' ? null : database === 'postgres' ? 'Postgres' : database === 'sqlite' ? 'SQLite' : database;
}

/** "41s", "1m 05s": how long a build took. */
export function took(seconds: number): string {
	return seconds < 60 ? `${seconds}s` : `${Math.floor(seconds / 60)}m ${String(seconds % 60).padStart(2, '0')}s`;
}

/** "Trian (owner), you": who can open it, its owner first, at most three names. */
export function peopleLine(people: NonNullable<ProjectAbout['people']>): string {
	const sorted = [...people].sort((a, b) => Number(b.owner) - Number(a.owner));
	const named = sorted.map((person) => (person.me ? (person.owner ? m.projects_people_you_owner() : m.projects_people_you()) : person.owner ? m.projects_people_owner({ name: person.name }) : person.name));
	const shown = named.slice(0, 3).join(', ');
	return named.length > 3 ? m.projects_people_more({ names: shown, count: named.length - 3 }) : shown;
}

/** The panel's "What's in this project": `about` undefined while it's read, null when it couldn't be. */
export function panelParts(card: ProjectCardData, about: ProjectAbout | null | undefined): ProjectParts {
	const checking = m.projects_part_checking();
	const part = <T>(value: T | null | undefined, say: (value: T) => string) => (about === undefined ? checking : value === null || value === undefined ? m.projects_part_unknown() : say(value));
	return {
		code: part(about?.files, (files) => [files === 1 ? m.projects_files_one() : m.projects_files({ count: files }), ...(about?.stack ? [STACKS[about.stack]?.() ?? about.stack] : [])].join(' · ')),
		url: card.liveUrl ?? m.projects_part_url_none(),
		data: part(about?.database, (database) => databaseName(database) ?? m.projects_part_none()),
		deploys: part(about?.deploys, (deploys) =>
			deploys.count === 0 ? m.projects_part_none() : deploys.builtIn === null ? String(deploys.count) : m.projects_part_deploys_built({ count: deploys.count, time: took(deploys.builtIn) }),
		),
		members: part(about?.people, (people) => (people.length ? peopleLine(people) : m.projects_people_you())),
	};
}

/** "live · deploy #14 · 2d ago" beside the live dot, for a project with an address. */
export function liveLine(card: ProjectCardData, about: ProjectAbout | null | undefined, now: Date): string | undefined {
	if (!card.liveUrl) return undefined;
	const number = about?.deploys?.liveNumber ?? card.deploys;
	const when = about?.deploys?.liveAt;
	return [m.projects_live_word(), m.projects_deploy({ number }), ...(when ? [m.projects_ago({ when: ago(when, now) })] : [])].join(' · ');
}

/** The panel's chats: the project's own chat (its workspace) and those moved into it, newest first. */
export function projectChats(rows: ProjectChatRow[], card: ProjectCardData, now: Date): ProjectChat[] {
	return rows
		.filter((row) => row.project === card.id)
		.sort((a, b) => Date.parse(b.updated_at) - Date.parse(a.updated_at))
		.map((row) => ({ title: row.title?.trim() || (row.own ? m.projects_chat_own() : m.projects_chat_untitled()), age: ago(row.updated_at, now), href: row.own ? card.href : `/chat/${row.id}` }));
}

const ROLE: Record<OrgRole, () => string> = { owner: m.workspace_role_owner, admin: m.workspace_role_admin, member: m.workspace_role_member };
const TONES: CrewId[] = ['tara', 'koda', 'maya', 'dana'];
const people = (count: number) => (count === 1 ? m.workspace_only_you() : m.workspace_members({ count }));

/** The Move dialog's workspaces: where it is now (shown, not chosen), then where it can go. */
export function moveTargets(choices: MoveChoices): MoveTarget[] {
	const here: MoveTarget[] = choices.current ? [{ id: choices.current.id, name: choices.current.name, meta: m.projects_move_current({ people: people(choices.current.members) }), tone: 'nomi', members: choices.current.members, current: true }] : [];
	return [
		...here,
		...choices.targets.map((target, index) => ({
			id: target.id,
			name: target.name,
			meta: [...(target.role ? [ROLE[target.role]()] : []), ...(target.members !== null ? [people(target.members)] : [])].join(' · '),
			tone: TONES[index % TONES.length],
			members: target.members ?? 0,
			...(target.owner && { owner: target.owner }),
		})),
	];
}
