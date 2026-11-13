import { describe, expect, it } from 'vitest';
import type { CrewMember, OrgList, SessionSummary, UsageBrief } from './api';
import { accountLine, chatsCount, crewRows, crewSummary, firstName, formatTokens, personName, projectsWorkspace, recents, usageCard, workspaceRows } from './shell';

const now = new Date('2026-10-21T08:14:30Z');

describe('personName and firstName', () => {
	it('use the name the person gave, else their username, else their address', () => {
		expect(personName({ display_name: 'Mario Prasetyo', username: 'mario', email: 'mario@nomi.test', avatar_url: null })).toBe('Mario Prasetyo');
		expect(personName({ display_name: '  ', username: 'mario', email: 'mario@nomi.test', avatar_url: null })).toBe('mario');
		expect(personName({ display_name: null, username: null, email: 'dinda.r@nomi.test', avatar_url: null })).toBe('dinda.r');
		expect(firstName('Mario Prasetyo')).toBe('Mario');
	});
});

describe('formatTokens', () => {
	it('writes token counts as the pen does: 21.8K, 1M', () => {
		expect([0, 950, 1000, 21_800, 21_849, 999_999, 1_000_000, 2_500_000].map(formatTokens)).toEqual(['0', '950', '1K', '21.8K', '21.8K', '1M', '1M', '2.5M']);
	});
});

const brief: UsageBrief = { plan: { id: 'free', name: 'Free', monthly_tokens: 1_000_000, custom_quota: false }, month: '2026-10', tokens_used: 21_800, can_upgrade: true };

describe('usageCard', () => {
	it('is the plan, the share used and the real figure', () => {
		expect(usageCard(brief)).toEqual({ plan: 'Free plan', used: 0.0218, caption: '21.8K / 1M tokens this month' });
	});

	it('never shows more than full, and nothing used of no allowance', () => {
		expect(usageCard({ ...brief, tokens_used: 3_000_000 }).used).toBe(1);
		expect(usageCard({ ...brief, plan: { ...brief.plan, monthly_tokens: 0 } }).used).toBe(0);
	});
});

describe('accountLine', () => {
	it('is the account menu’s plan and usage, in mono', () => {
		expect(accountLine(brief)).toBe('Free · 21.8K / 1M tokens');
	});
});

const session = (over: Partial<SessionSummary>): SessionSummary => ({
	id: 's1',
	title: 'Budget belanja Oktober',
	last_message: null,
	updated_at: '2026-10-21T06:00:00Z',
	project_id: null,
	pinned: false,
	in_project_id: null,
	...over,
});

describe('recents', () => {
	const sessions = [
		session({ id: 'a', title: 'Starting a new conversation', last_message: { content: 'hi', created_at: '2026-10-21T08:14:00Z' } }),
		session({ id: 'b', title: 'salah-daily-tracker', project_id: 'p1', updated_at: '2026-10-19T08:00:00Z' }),
		session({ id: 'c', title: null, updated_at: '2026-10-21T06:00:00Z' }),
		session({ id: 'd', title: 'Invoice PT Sinar Jaya', updated_at: '2026-10-18T08:00:00Z' }),
		session({ id: 'e', title: 'Too old to show', updated_at: '2026-10-01T08:00:00Z' }),
	];

	it('keeps a pinned chat on top, even an older one', () => {
		const pinned = [...sessions.slice(0, 4), session({ id: 'e', title: 'Pinned rent', updated_at: '2026-10-01T08:00:00Z', pinned: true, pinned_at: '2026-10-20T08:00:00Z' })];
		expect(recents(pinned, now).map((recent) => recent.href)).toEqual(['/chat/e', '/chat/a', '/projects/p1', '/chat/c']);
	});

	it('lists the latest four chats and projects, with how long ago', () => {
		expect(recents(sessions, now)).toEqual([
			{ href: '/chat/a', kind: 'chat', title: 'Starting a new conversation', when: 'now' },
			{ href: '/projects/p1', kind: 'project', title: 'salah-daily-tracker', when: '2d' },
			{ href: '/chat/c', kind: 'chat', title: 'New chat', when: '2h' },
			{ href: '/chat/d', kind: 'chat', title: 'Invoice PT Sinar Jaya', when: '3d' },
		]);
	});

	it('marks the one on screen', () => {
		expect(recents(sessions, now, '/chat/c').find((item) => item.href === '/chat/c')?.active).toBe(true);
	});

	it('counts chats, not projects’ own chats', () => {
		expect(chatsCount(sessions)).toBe(4);
	});
});

const crew = (over: Partial<CrewMember>): CrewMember => ({ agent_type: 'chitchat', name: 'Nomi', role: 'Talks with you and routes the work', is_dynamic: false, state: 'idle', status: '', ...over });

describe('crewRows', () => {
	it('gives each built-in agent its mascot, and says only what is not idle', () => {
		expect(
			crewRows([
				crew({}),
				crew({ agent_type: 'money', name: 'Dana', role: 'Transactions, budgets, subscriptions', state: 'working' }),
				crew({ agent_type: 'planning', name: 'Rena', role: 'Plans', state: 'waiting' }),
				crew({ agent_type: 'inspector', name: 'Iris', role: 'Looks into issues' }),
			]),
		).toEqual([
			{ agent: 'nomi', name: 'Nomi', role: 'Talks with you and routes the work' },
			{ agent: 'dana', name: 'Dana', role: 'Transactions, budgets, subscriptions', state: 'working' },
			{ agent: 'rena', name: 'Rena', role: 'Plans', state: 'waiting on you' },
			{ name: 'Iris', role: 'Looks into issues' },
		]);
	});

	it('maps every crew member the design has a mascot for', () => {
		const types = ['chitchat', 'money', 'reminders', 'files', 'planning', 'coding', 'workspace', 'health', 'personality', 'supervisor'];
		expect(crewRows(types.map((agent_type) => crew({ agent_type }))).map((row) => row.agent)).toEqual(['nomi', 'dana', 'kala', 'maya', 'rena', 'koda', 'tara', 'raga', 'personality', 'supervisor']);
	});
});

describe('crewSummary', () => {
	it('says how many are idle, or how many are working', () => {
		expect(crewSummary(Array.from({ length: 10 }, () => crew({})))).toEqual({ kind: 'idle', label: '10 idle' });
		expect(crewSummary([crew({ state: 'working' }), crew({ state: 'waiting' }), crew({}), crew({ state: 'done' })])).toEqual({ kind: 'live', label: '2 working · 2 idle' });
	});
});

const orgs: OrgList = {
	orgs: [
		{ id: 'o1', name: 'Mario', role: 'owner', members: 1, projects: 1 },
		{ id: 'o2', name: "Trian's space", role: 'admin', members: 3, projects: 4 },
		{ id: 'o3', name: 'Kopi Kita', role: 'member', members: 2, projects: 2 },
	],
	project_org_id: 'o2',
};

describe('workspaceRows', () => {
	it('lists each workspace with its tile, role, people and projects, checking the one for projects', () => {
		expect(workspaceRows(orgs)).toEqual([
			{ id: 'o1', name: 'Mario', tone: 'plain', meta: 'Owner · only you · 1 project', chosen: false },
			{ id: 'o2', name: "Trian's space", tone: 'teal', meta: 'Admin · 3 members · 4 projects', chosen: true },
			{ id: 'o3', name: 'Kopi Kita', tone: 'amber', meta: 'Member · 2 members · 2 projects', chosen: false },
		]);
	});
});

describe('projectsWorkspace', () => {
	it('is the workspace projects are listed from', () => {
		expect(projectsWorkspace(orgs)?.name).toBe("Trian's space");
	});

	it('is nothing until one is picked, or when that one is gone', () => {
		expect(projectsWorkspace({ ...orgs, project_org_id: null })).toBeNull();
		expect(projectsWorkspace({ ...orgs, project_org_id: 'gone' })).toBeNull();
	});
});
