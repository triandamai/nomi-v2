// "@" in the composer: which crew member (or project file) the person is naming, and the text once
// they pick one. The server routes a message naming a crew member to them (nomi_agent_core::mention),
// so what goes in is the plain "@Name" anyone could type.
import type { MentionOption } from '$lib/components/ds/chat/mention';
import { crewId } from '$lib/crewId';
import type { CrewRosterMember } from '$lib/crew';
import { toolDescription } from '$lib/toolPicker';
import type { CatalogTool } from '$lib/types';

/** A tool of the catalog offered by "@" in a soul editor; `keywords` also find it, `group` says
 * where it is from. */
export interface ToolMentionOption {
	kind: 'tool';
	name: string;
	group: string;
	description: string;
	keywords?: string[];
}

/** What "@" can offer: the chat's options, and tools in a soul editor. */
export type MentionChoice = MentionOption | ToolMentionOption;

/** The order the pickers list the kinds in; an option's place in `active` follows it. */
const MENTION_KIND_ORDER: MentionChoice['kind'][] = ['crew', 'file', 'tool'];

/** How many options the picker lists at once. */
export const MAX_MENTION_OPTIONS = 8;

/** The "@query" being typed at `caret`: where its "@" is and what follows it. */
export function mentionAt(text: string, caret: number): { start: number; query: string } | null {
	const before = text.slice(0, caret);
	const match = /(^|[\s([{"'])@([\p{L}\p{N}_.\-/]*)$/u.exec(before);
	if (!match) return null;
	return { start: caret - match[2].length - 1, query: match[2] };
}

/** Whether a tool option's keywords hold `query`: tools are also found by what they're about. */
function keywordMatch(option: MentionChoice, query: string): boolean {
	return option.kind === 'tool' && (option.keywords ?? []).some((keyword) => keyword.toLowerCase().includes(query));
}

/** The options whose name (or, for a file, its path) starts with `query`, then those that contain
 * it (a tool also by its keywords). Listed by kind, as the picker shows them (crew, files, tools),
 * so the highlighted position is the position in this list. */
export function filterMentions<T extends MentionChoice>(options: T[], query: string): T[] {
	const q = query.toLowerCase();
	const name = (option: MentionChoice) => option.name.toLowerCase();
	const starts = options.filter((option) => name(option).startsWith(q));
	const contains = options.filter((option) => !name(option).startsWith(q) && name(option).includes(q));
	const about = options.filter((option) => !name(option).includes(q) && keywordMatch(option, q));
	const found = [...starts, ...contains, ...about];
	const byKind = MENTION_KIND_ORDER.flatMap((kind) => found.filter((option) => option.kind === kind));
	return byKind.slice(0, MAX_MENTION_OPTIONS);
}

/** The crew as "@" options, by the names the server routes on. */
export function crewMentionOptions(roster: CrewRosterMember[]): MentionOption[] {
	return roster.map((member) => ({ kind: 'crew', agent: crewId(member.agent_type), name: member.name, role: member.role }));
}

/** The catalog's tools as "@" options, described in the person's language. */
export function toolMentionOptions(tools: CatalogTool[]): ToolMentionOption[] {
	return tools.map((tool) => ({ kind: 'tool', name: tool.name, group: tool.group, description: toolDescription(tool), keywords: tool.keywords }));
}

/** `text` with the "@query" from `start` to `caret` replaced by the picked name and a space. */
export function insertMention(text: string, start: number, caret: number, name: string): { text: string; caret: number } {
	const token = `@${name} `;
	const after = text.slice(caret).replace(/^ /, '');
	return { text: text.slice(0, start) + token + after, caret: start + token.length };
}
