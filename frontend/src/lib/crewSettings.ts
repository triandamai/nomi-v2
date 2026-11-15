// Settings › Crew: a member's permissions, its soul.md and a new member of one's own
// (/api/agents/:type/permissions, /soul, POST /api/agents). Limits mirror
// backend/crates/nomi-server/src/routes/crew_settings.rs.

import type { CrewActivityItem, CrewPermissionLevel } from './types';

export const PERMISSION_LEVELS: CrewPermissionLevel[] = ['allow', 'ask', 'never'];
export const SOUL_MAX_CHARS = 8_000;
export const NAME_MAX_CHARS = 40;
export const ROLE_MAX_CHARS = 200;

const LEVEL_FIELD = 'level:';
/** A hidden field that marks a form as carrying the `tools` list, even an empty one. */
export const TOOLS_SET_FIELD = 'tools_set';

/** The form field a tool's level is posted in. */
export function levelField(key: string): string {
	return `${LEVEL_FIELD}${key}`;
}

/** The permissions form as the PUT body: every `level:<key>` field with a known level. */
export function levelsFromForm(data: FormData): { key: string; level: CrewPermissionLevel }[] {
	const levels: { key: string; level: CrewPermissionLevel }[] = [];
	for (const [name, value] of data.entries()) {
		if (!name.startsWith(LEVEL_FIELD)) continue;
		const level = String(value) as CrewPermissionLevel;
		if (PERMISSION_LEVELS.includes(level)) levels.push({ key: name.slice(LEVEL_FIELD.length), level });
	}
	return levels;
}

export interface NewMemberBody {
	template?: string;
	name?: string;
	role?: string;
	soul?: string;
	tools?: string[];
}

/** The New member form as the POST body, or which field is missing or too long. A template fills
 * whatever is left empty. */
export function newMemberFromForm(data: FormData): { body: NewMemberBody } | { invalid: 'name' | 'role' | 'soul' } {
	const read = (field: string) => String(data.get(field) ?? '').trim();
	const template = read('template');
	const name = read('name');
	const role = read('role');
	const soul = read('soul');
	if ((!template && !name) || name.length > NAME_MAX_CHARS) return { invalid: 'name' };
	if ((!template && !role) || role.length > ROLE_MAX_CHARS) return { invalid: 'role' };
	if (soul.length > SOUL_MAX_CHARS) return { invalid: 'soul' };
	const body: NewMemberBody = {};
	if (template) body.template = template;
	if (name) body.name = name;
	if (role) body.role = role;
	if (soul) body.soul = soul;
	// The form always says whether it carries the tools field, so an emptied list is sent as empty
	// (and not mistaken for "keep the template's").
	if (data.has(TOOLS_SET_FIELD)) body.tools = toolsFromForm(data);
	return { body };
}

/** "list_transactions" → "List transactions". */
export function toolLabel(name: string): string {
	const words = name.replace(/_/g, ' ').trim();
	return words ? words[0].toUpperCase() + words.slice(1) : words;
}

/** Whether an Activity line went wrong (a failed task or a tool error). */
export function activityFailed(item: CrewActivityItem): boolean {
	return item.status === 'failed' || item.status === 'error';
}

/** The tool names in a form's repeated `tools` field, once each, in order. */
export function toolsFromForm(data: FormData, field = 'tools'): string[] {
	const names = data.getAll(field).map((value) => String(value).trim());
	return [...new Set(names.filter(Boolean))];
}
