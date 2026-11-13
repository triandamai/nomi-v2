import { describe, expect, it } from 'vitest';
import { answersText, checklist, replyBlocks } from './blocks';

const reply = (blocks: unknown[], extra: Record<string, unknown> = {}) => ({ id: 'm1', agent_display_name: 'Dana', content_blocks: blocks, sources: [], used_memories: [], saved_memories: [], ...extra }) as never;
const kinds = (message: never) => replyBlocks(message, { language: 'id' }).map((block) => block.kind);

describe('reply blocks', () => {
	it('turns clarifying questions into the clarify card, from the crew member who asked', () => {
		const [block] = replyBlocks(reply([{ kind: 'questions', intro: 'Two things first', questions: [{ question: 'Which month?', options: [{ label: 'October', detail: 'so far' }, { label: 'September' }], multiple: false }] }]), { language: 'en' });
		expect(block).toMatchObject({
			kind: 'clarify',
			agent: 'dana',
			title: 'Two things first',
			questions: [{ topic: 'Question 1', title: 'Which month?', options: [{ title: 'October', detail: 'so far' }, { title: 'September' }], multiple: false }],
		});
	});

	it('writes the answers as the person would: each question and what was picked', () => {
		expect(answersText([{ question: 'Which month?' }, { question: 'Split by?' }], [['October'], []])).toBe('Which month? October');
		expect(answersText([{ question: 'Shops?' }], [['Indomaret', 'Alfamart']])).toBe('Shops? Indomaret, Alfamart');
	});

	it('keeps an approval’s decision, and shows its input as fields', () => {
		const [block] = replyBlocks(reply([{ kind: 'approval_request', id: 'ap1', tool_name: 'send_email', description: 'Send the invoice to Bu Siti', input: { to: 'siti@kopikita.id', amount: 150000 }, status: 'approved', decided_at: '2026-11-11T05:00:00Z' }]), { language: 'en' });
		expect(block).toMatchObject({ kind: 'approval', title: 'Send the invoice to Bu Siti', status: 'approved', fields: [{ key: 'to', value: 'siti@kopikita.id' }, { key: 'amount', value: '150000' }] });
	});

	it('reads a plan’s steps from its checklist', () => {
		expect(checklist('# Plan\n\n- [x] Read the receipts\n- [ ] Sum by category\n  - [ ] nested is a step too\nnot a step')).toEqual([
			{ text: 'Read the receipts', done: true },
			{ text: 'Sum by category', done: false },
			{ text: 'nested is a step too', done: false },
		]);
	});

	it('shows a table with its title and source, numbers in the person’s format', () => {
		const [block] = replyBlocks(reply([{ kind: 'table', variant: 'data', title: 'October', source: 'BCA statement', columns: [{ key: 'cat', label: 'Category' }, { key: 'sum', label: 'Spent' }], rows: [{ cat: 'Food', sum: 1250000 }] }]), { language: 'id' });
		expect(block).toMatchObject({ kind: 'table', title: 'October', source: 'BCA statement', columns: ['Category', 'Spent'], rows: [['Food', '1.250.000']] });
	});

	it('maps each view the design system draws, and says which it doesn’t', () => {
		const views = [
			{ view: 'decision', question: 'Which bank?', options: [{ label: 'BCA', detail: 'no fee', recommended: true }, { label: 'Mandiri' }] },
			{ view: 'form', title: 'Your address', fields: [{ key: 'street', label: 'Street' }], submit_label: 'Save' },
			{ view: 'selection', question: 'Which to cancel?', items: [{ name: 'Netflix', amount: 186000, selected: true }] },
			{ view: 'cart', title: 'Groceries', store: 'Sayurbox', items: [{ name: 'Telur', quantity: 2, price: 29000 }], delivery: 10000, checkout_url: 'https://sayurbox.com/c/1' },
			{ view: 'sources', items: [{ title: 'Pajak', url: 'https://pajak.go.id/x', kind: 'web' }] },
			{ view: 'link', url: 'https://kopikita.id/menu', title: 'Menu' },
			{ view: 'code', code: 'npm run build', language: 'sh' },
			{ view: 'boundary', title: 'I can’t move money', reason: 'Payments stay with you.', alternatives: [{ title: 'Draft the transfer', agent: 'Dana' }] },
			{ view: 'quick_replies', replies: ['Yes', 'No'] },
			{ view: 'suggestions', items: ['Do the same for September'] },
			{ view: 'chart', labels: ['a'], series: [] },
		];
		expect(kinds(reply(views.map((view) => ({ kind: 'view', ...view }))))).toEqual(['decision', 'form', 'selection', 'cart', 'sources', 'link', 'code', 'boundary', 'replies', 'replies', 'unsupported']);
		const cart = replyBlocks(reply([{ kind: 'view', ...views[3] }]), { language: 'id' })[0];
		expect(cart).toMatchObject({ kind: 'cart', subtotal: '58.000', delivery: '10.000', total: '68.000', checkoutHref: 'https://sayurbox.com/c/1' });
	});

	it('shows reminders, connect prompts, a stop, voice, and what the reply leaned on', () => {
		const message = reply(
			[
				{ kind: 'reminder', reminder_id: 'r1', title: 'Bayar listrik', notes: null, due_at: '2026-11-11T09:00:00Z', recurrence: null },
				{ kind: 'workspace_connect', reason: 'not_connected', services: ['gmail'], agent_type: 'workspace' },
				{ kind: 'stopped', tasks: [{ agent: 'Dana', title: 'Summing October', state: 'stopped' }] },
				{ kind: 'voice', url: '/voice/u1/a.mp3', text: 'Halo', voice: 'Sarah' },
			],
			{ sources: [{ title: 'BCA', url: 'https://bca.co.id' }], used_memories: [{ id: 'f1', content: 'Paid on the 5th', kind: 'fact' }] },
		);
		expect(kinds(message)).toEqual(['reminder', 'connect', 'stopped', 'voice', 'sources', 'memory_used']);
	});

	it('leaves out what the thread already handles: thinking, failures, quota and follow-ups', () => {
		expect(kinds(reply([{ kind: 'reasoning', text: '…' }, { kind: 'run_failed' }, { kind: 'quota_notice', reason: 'x' }, { kind: 'suggestions', items: ['x'] }]))).toEqual([]);
	});
});
