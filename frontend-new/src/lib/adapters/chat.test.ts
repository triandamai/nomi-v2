import { describe, expect, it } from 'vitest';
import type { CrewMember, HomeSummary, ModelOptions, SessionSummary } from './api';
import { chatCrewRows, chatListGroups, crewIdOf, failureView, modelMenu, starters, todayColumns, withMention, mentionOptions } from './chat';

const member = (agent_type: string, name: string, role = ''): CrewMember => ({ agent_type, name, role, is_dynamic: false, state: 'idle', status: '' });
const crew = [member('chitchat', 'Nomi', 'Talks with you'), member('money', 'Dana', 'Transactions'), member('planning', 'Rena', 'Plans'), member('coding', 'Koda', 'Builds'), member('reminders', 'Kala', 'Reminds you')];

describe('crewIdOf', () => {
	it('knows each crew member by the name their messages carry', () => {
		expect(crewIdOf('Dana')).toBe('dana');
		expect(crewIdOf('supervisor')).toBe('supervisor');
		expect(crewIdOf(null)).toBe('nomi');
		expect(crewIdOf('Iris')).toBe('nomi');
	});
});

describe('starters', () => {
	it('routes each to the crew member who owns it, and leaves out one whose owner is missing', () => {
		expect(starters(crew).map((starter) => [starter.agent, starter.route])).toEqual([
			['dana', '→ dana'],
			['rena', '→ rena'],
			['koda', '→ koda'],
			['kala', '→ kala'],
		]);
		expect(starters(crew.filter((one) => one.name !== 'Koda'))).toHaveLength(3);
	});
});

describe('mentions', () => {
	it('offers the crew the design draws to mention, Nomi first', () => {
		const rows = [{ agent: 'nomi' as const, name: 'Nomi', role: 'Talks with you' }, { name: 'Iris', role: 'Inspects' }];
		expect(mentionOptions(rows)).toEqual([{ kind: 'crew', agent: 'nomi', name: 'Nomi', role: 'Talks with you' }]);
	});

	it('says in the crew panel who replied in this chat and who is answering', () => {
		const rows = [{ agent: 'nomi' as const, name: 'Nomi', role: 'Talks with you' }, { agent: 'dana' as const, name: 'Dana', role: 'Transactions' }];
		expect(chatCrewRows(rows, { replied: ['Nomi'], working: 'Dana' })).toEqual([
			{ agent: 'nomi', name: 'Nomi', role: 'Replied in this chat' },
			{ agent: 'dana', name: 'Dana', role: 'Transactions', state: 'working' },
		]);
	});

	it('addresses a message to a crew member unless it already is, or it is for Nomi', () => {
		expect(withMention('How much on food?', 'Dana')).toBe('@Dana How much on food?');
		expect(withMention('@Dana How much?', 'Dana')).toBe('@Dana How much?');
		expect(withMention('Hi', 'Nomi')).toBe('Hi');
	});
});

describe('todayColumns', () => {
	const empty: HomeSummary = { since: '', timezone: 'UTC', while_you_were_out: [], today: [], plans: [], while_you_were_out_total: 0, today_total: 0, plans_total: 0 };

	it('says all is quiet when there is nothing', () => {
		const columns = todayColumns(empty, 'UTC');
		expect(columns.map((column) => [column.key, column.count])).toEqual([
			['out', 0],
			['reminders', 0],
			['plans', 0],
		]);
		expect(columns[0].body).toBe('All quiet. Nothing new from the crew since you last looked.');
	});

	it('names the latest of each when there is something', () => {
		const columns = todayColumns(
			{
				...empty,
				while_you_were_out: [{ kind: 'finished', agent: 'Dana', title: 'Budget check', detail: 'Done', session_id: 's1', at: '2026-10-21T01:00:00Z' }],
				while_you_were_out_total: 2,
				today: [{ id: 'r', run_at: '2026-10-21T09:00:00Z', label: 'Stretch', agent: 'Kala', recurrence: null }],
				today_total: 1,
				plans: [{ kind: 'plan', title: 'Bandung weekend', agent: 'Rena', done: 2, total: 5, session_id: 's2', updated_at: '' }],
				plans_total: 1,
			},
			'Asia/Jakarta',
		);
		expect(columns.map((column) => column.count)).toEqual([2, 1, 1]);
		expect(columns[0].body).toBe('Dana: Budget check');
		expect(columns[1].body).toBe('Stretch · 16:00');
		expect(columns[2].body).toBe('Bandung weekend · 2 of 5 done');
		expect(columns[2].action).toEqual({ label: 'Open the plan', href: '/chat/s2' });
		expect(columns[1].action).toEqual({ label: 'Add reminder', prompt: '@Kala Remind me to ' });
	});
});

describe('chatListGroups', () => {
	const now = new Date('2026-10-21T08:00:00Z');
	const session = (id: string, at: string, extra: Partial<SessionSummary> = {}): SessionSummary => ({
		id,
		title: id,
		last_message: { content: `said in ${id}`, created_at: at },
		updated_at: at,
		project_id: null,
		pinned: false,
		in_project_id: null,
		...extra,
	});

	it('puts pinned chats first, then the rest by day, leaving projects’ own chats out', () => {
		const groups = chatListGroups(
			[
				session('a', '2026-10-21T07:00:00Z'),
				session('b', '2026-10-20T07:00:00Z', { pinned: true, pinned_at: '2026-10-20T08:00:00Z' }),
				session('c', '2026-10-20T06:00:00Z'),
				session('p', '2026-10-20T06:00:00Z', { project_id: 'proj' }),
				session('d', '2026-10-01T06:00:00Z', { title: null }),
			],
			{ now, timeZone: 'UTC', language: 'en', query: '' },
		);
		expect(groups.map((group) => [group.label, group.rows.map((row) => row.id)])).toEqual([
			['Pinned', ['b']],
			['Today', ['a']],
			['Yesterday', ['c']],
			['Thu 1 Oct', ['d']],
		]);
		expect(groups[1].rows[0]).toMatchObject({ title: 'a', preview: 'said in a', time: '07:00', href: '/chat/a', pinned: false });
		expect(groups[3].rows[0].title).toBe('New chat');
	});

	it('finds chats by what they say', () => {
		const groups = chatListGroups([session('rent', '2026-10-21T07:00:00Z'), session('food', '2026-10-21T06:00:00Z')], { now, timeZone: 'UTC', language: 'en', query: 'rent' });
		expect(groups.flatMap((group) => group.rows.map((row) => row.id))).toEqual(['rent']);
	});
});

describe('failureView', () => {
	it('names each failure and offers the move that fixes it', () => {
		expect(failureView('no_credits', { retry: true, canUseCredits: true })).toEqual({ title: 'Out of credits', actions: ['retry', 'credits'] });
		expect(failureView('no_credits', { retry: true, canUseCredits: false })).toEqual({ title: 'Out of credits', actions: ['retry', 'models'] });
		expect(failureView('quota', { retry: false, canUseCredits: false })).toEqual({ title: 'Out of Nomi credits', actions: ['upgrade', 'own_key'] });
		expect(failureView('too_long', { retry: true, canUseCredits: false })).toEqual({ title: 'Context limit', actions: ['new_chat', 'models'] });
		expect(failureView('rate_limited', { retry: true, canUseCredits: false })).toEqual({ title: 'Rate limited', actions: ['retry', 'models'] });
		expect(failureView('other', { retry: false, canUseCredits: false })).toEqual({ title: 'Couldn’t reply', actions: [] });
	});
});

describe('modelMenu', () => {
	const options: ModelOptions = {
		current: null,
		groups: [
			{ id: 'nomi:DeepSeek', source: 'nomi', provider_id: 'deepseek', has_logo: true, label: 'DeepSeek', connection_id: null, error: null, models: [{ key: 'admin:1', label: 'deepseek-flash', model_id: 'deepseek-chat', free: true, enabled: true }] },
			{
				id: 'conn:c1',
				source: 'connection',
				provider_id: 'anthropic',
				has_logo: true,
				label: 'Anthropic',
				connection_id: 'c1',
				error: null,
				models: [
					{ key: 'conn:c1:claude-sonnet', label: 'claude-sonnet', model_id: 'claude-sonnet', free: false, enabled: true },
					{ key: 'conn:c1:off', label: 'hidden', model_id: 'off', free: false, enabled: false },
				],
			},
		],
	};

	it('groups the models by who pays, your keys first, and marks the one in use', () => {
		expect(modelMenu(options, 'admin:1')).toEqual([
			{ group: 'Your keys', models: [{ key: 'conn:c1:claude-sonnet', label: 'claude-sonnet', source: 'Anthropic', chosen: false }] },
			{ group: 'Nomi credits', models: [{ key: 'admin:1', label: 'deepseek-flash', source: 'Nomi credits', chosen: true }] },
		]);
	});

	it('is nothing to pick from when there are no models', () => {
		expect(modelMenu({ groups: [], current: null }, null)).toEqual([]);
	});
});
