import { describe, expect, it } from 'vitest';
import { activityFailed, levelField, levelsFromForm, newMemberFromForm, toolLabel, toolsFromForm, TOOLS_SET_FIELD } from './crewSettings';
import type { CrewActivityItem } from './types';

const form = (entries: [string, string][]) => {
	const data = new FormData();
	for (const [k, v] of entries) data.append(k, v);
	return data;
};

describe('levelsFromForm', () => {
	it('reads each tool level and skips other fields and unknown levels', () => {
		const data = form([
			[levelField('log_transaction'), 'never'],
			[levelField('list_budgets'), 'allow'],
			[levelField('set_budget'), 'maybe'],
			['agent_type', 'money'],
		]);
		expect(levelsFromForm(data)).toEqual([
			{ key: 'log_transaction', level: 'never' },
			{ key: 'list_budgets', level: 'allow' },
		]);
	});
});

describe('newMemberFromForm', () => {
	it('needs a name and a role from scratch', () => {
		expect(newMemberFromForm(form([['role', 'Cooks']]))).toEqual({ invalid: 'name' });
		expect(newMemberFromForm(form([['name', 'Chef']]))).toEqual({ invalid: 'role' });
		expect(newMemberFromForm(form([['name', ' Chef '], ['role', 'Cooks'], ['soul', '']]))).toEqual({ body: { name: 'Chef', role: 'Cooks' } });
	});

	it('lets a template fill the rest', () => {
		expect(newMemberFromForm(form([['template', 'budget_coach']]))).toEqual({ body: { template: 'budget_coach' } });
		expect(newMemberFromForm(form([['template', 'budget_coach'], ['name', 'Penny']]))).toEqual({ body: { template: 'budget_coach', name: 'Penny' } });
	});

	it('refuses what is too long', () => {
		expect(newMemberFromForm(form([['name', 'x'.repeat(41)], ['role', 'r']]))).toEqual({ invalid: 'name' });
		expect(newMemberFromForm(form([['name', 'n'], ['role', 'r'], ['soul', 'x'.repeat(8_001)]]))).toEqual({ invalid: 'soul' });
	});
});

describe('toolLabel', () => {
	it('reads a tool name as words', () => {
		expect(toolLabel('list_transactions')).toBe('List transactions');
		expect(toolLabel('')).toBe('');
	});
});

describe('activityFailed', () => {
	const item = (status: string | null): CrewActivityItem => ({ kind: 'task', text: 't', status, session_id: null, chat_title: null, at: '2026-11-15T00:00:00Z' });
	it('is true for failed tasks and tool errors', () => {
		expect(activityFailed(item('failed'))).toBe(true);
		expect(activityFailed(item('error'))).toBe(true);
		expect(activityFailed(item('completed'))).toBe(false);
		expect(activityFailed(item(null))).toBe(false);
	});
});

describe('tools in forms', () => {
	it('reads each tool once, in order, skipping blanks', () => {
		expect(toolsFromForm(form([['tools', 'a'], ['tools', ' b '], ['tools', 'a'], ['tools', '']]))).toEqual(['a', 'b']);
	});

	it('sends the tools of a new member only when the form carries the field, even empty', () => {
		const base: [string, string][] = [['name', 'Chef'], ['role', 'Cooks']];
		expect(newMemberFromForm(form(base))).toEqual({ body: { name: 'Chef', role: 'Cooks' } });
		expect(newMemberFromForm(form([...base, [TOOLS_SET_FIELD, '1']]))).toEqual({ body: { name: 'Chef', role: 'Cooks', tools: [] } });
		expect(newMemberFromForm(form([...base, [TOOLS_SET_FIELD, '1'], ['tools', 'read_file']]))).toEqual({ body: { name: 'Chef', role: 'Cooks', tools: ['read_file'] } });
	});
});
