import { describe, expect, it } from 'vitest';
import { crewMentionOptions, type MentionChoice, type ToolMentionOption, filterMentions, insertMention, mentionAt, toolMentionOptions, MAX_MENTION_OPTIONS } from './mentions';
import type { CrewRosterMember } from './crew';
import type { MentionOption } from '$lib/components/ds/chat/mention';

const crew: MentionOption[] = [
	{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money' },
	{ kind: 'crew', agent: 'kala', name: 'Kala', role: 'Reminders' },
	{ kind: 'crew', agent: 'koda', name: 'Koda', role: 'Code' },
];

describe('mentionAt', () => {
	it('finds the "@query" right before the caret', () => {
		expect(mentionAt('@da', 3)).toEqual({ start: 0, query: 'da' });
		expect(mentionAt('ask @Ko', 7)).toEqual({ start: 4, query: 'Ko' });
		expect(mentionAt('hi @', 4)).toEqual({ start: 3, query: '' });
	});

	it('ignores an email address and a finished mention', () => {
		expect(mentionAt('dana@ex', 7)).toBeNull();
		expect(mentionAt('@Dana coffee', 12)).toBeNull();
	});
});

describe('filterMentions', () => {
	it('puts names starting with the query first', () => {
		expect(filterMentions(crew, 'k').map((o) => o.name)).toEqual(['Kala', 'Koda']);
		expect(filterMentions(crew, 'a').map((o) => o.name)).toEqual(['Dana', 'Kala', 'Koda']);
		expect(filterMentions(crew, 'zz')).toEqual([]);
	});

	it('lists a handful at most', () => {
		const many = Array.from({ length: 20 }, (_, i): MentionOption => ({ kind: 'file', name: `f${i}.ts`, where: 'src' }));
		expect(filterMentions(many, 'f')).toHaveLength(MAX_MENTION_OPTIONS);
	});
});

const tools: ToolMentionOption[] = [
	{ kind: 'tool', name: 'list_files', group: 'Coding', description: 'List project files', keywords: ['browse'] },
	{ kind: 'tool', name: 'read_file', group: 'Coding', description: 'Read a project file' },
];

describe('filterMentions with tools', () => {
	it('lists crew before tools whatever their order in, so the highlight follows what is shown', () => {
		expect(filterMentions<MentionChoice>([...tools, ...crew], 'k').map((o) => o.name)).toEqual(['Kala', 'Koda']);
		expect(filterMentions<MentionChoice>([...tools, ...crew], 'a').map((o) => o.name)).toEqual(['Dana', 'Kala', 'Koda', 'read_file']);
	});

	it('finds a tool by its keywords, after name matches', () => {
		expect(filterMentions(tools, 'browse').map((o) => o.name)).toEqual(['list_files']);
		expect(filterMentions(crew, 'browse')).toEqual([]);
	});
});

describe('mention option builders', () => {
	it('maps the roster to crew options', () => {
		const roster = [{ agent_type: 'money', name: 'Dana', role: 'Money' }] as CrewRosterMember[];
		expect(crewMentionOptions(roster)).toEqual([{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money' }]);
	});

	it('maps the catalog to tool options with the shown description', () => {
		const [option] = toolMentionOptions([{ name: 'brand_new_tool', description: 'API text', group: 'X', keywords: ['k'] }]);
		expect(option).toEqual({ kind: 'tool', name: 'brand_new_tool', group: 'X', description: 'API text', keywords: ['k'] });
	});
});

describe('insertMention', () => {
	it('swaps the query for the name and a space, caret after it', () => {
		expect(insertMention('ask @Ko now', 4, 7, 'Koda')).toEqual({ text: 'ask @Koda now', caret: 10 });
		expect(insertMention('@d', 0, 2, 'Dana')).toEqual({ text: '@Dana ', caret: 6 });
		expect(insertMention('use @read', 4, 9, 'read_file')).toEqual({ text: 'use @read_file ', caret: 15 });
	});
});
