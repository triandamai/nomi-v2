import { describe, expect, it } from 'vitest';
import type { HomeSummary } from '$lib/adapters/api';
import { needsYou, newNeedsYou } from './needsYou';

const summary = (items: HomeSummary['while_you_were_out']): HomeSummary => ({ since: '', timezone: 'Asia/Jakarta', while_you_were_out: items, today: [], plans: [], while_you_were_out_total: items.length, today_total: 0, plans_total: 0 });

describe('needs you', () => {
	it('keeps the approvals waiting on the person, each with its chat', () => {
		const items = needsYou(
			summary([
				{ kind: 'needs_you', agent: 'workspace', title: 'Tara needs your OK', detail: 'Send an email', session_id: 's1', at: '2026-11-18T07:00:00Z' },
				{ kind: 'finished', agent: 'money', title: 'Dana finished', detail: '', session_id: 's2', at: '2026-11-18T06:00:00Z' },
				{ kind: 'needs_you', agent: 'workspace', title: 'x', detail: '', session_id: null, at: '2026-11-18T05:00:00Z' },
			]),
		);
		expect(items).toEqual([{ key: 's1@2026-11-18T07:00:00Z', sessionId: 's1', agent: 'workspace', at: '2026-11-18T07:00:00Z' }]);
		expect(needsYou(null)).toEqual([]);
	});

	it('says only what’s new, and not for the chat that’s open', () => {
		const items = [
			{ key: 'a', sessionId: 's1', agent: 'workspace', at: '' },
			{ key: 'b', sessionId: 's2', agent: 'workspace', at: '' },
			{ key: 'c', sessionId: 's3', agent: 'workspace', at: '' },
		];
		expect(newNeedsYou(new Set(['a']), items, 's3').map((item) => item.key)).toEqual(['b']);
	});
});
