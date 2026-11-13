import { expect, test } from '@playwright/test';
import { api, createAccount, signInAs } from './support/accounts';
import { seedReply } from './support/seed';

// Rich replies against the real backend (Flow / Chat — Rich replies y2Qxt2, Clarify → plan S3yL5):
// what a reply carries beyond its text, drawn with the design system's cards, and acted on. The
// fake model writes only text, so the replies with blocks are put in the database as the crew's
// turn would; answering them goes through the real backend.

const REPLY = 'This is a fake response for local development and testing.';

async function chatAfter(text: string) {
	const account = await createAccount();
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text }) });
	return { account, sessionId: session_id };
}

test('a reply’s table, decision, code, boundary and a view the app can’t show yet are all drawn', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Rangkum belanja Oktober');
	await seedReply(sessionId, {
		content: 'Here’s October.',
		agent: 'Dana',
		blocks: [
			{ kind: 'table', variant: 'data', title: 'October spending', source: 'BCA statement', columns: [{ key: 'cat', label: 'Category' }, { key: 'sum', label: 'Spent' }], rows: [{ cat: 'Food', sum: 1250000 }] },
			{ kind: 'view', view: 'code', code: 'SUM(amount)', language: 'sql' },
			{ kind: 'view', view: 'boundary', title: 'I can’t move money', reason: 'Payments stay with you.' },
			{ kind: 'view', view: 'chart', labels: ['Food'], series: [{ name: 'Spent', values: [1] }] },
			{ kind: 'view', view: 'decision', question: 'Which budget next?', options: [{ label: 'November', recommended: true }, { label: 'December' }] },
		],
	});
	await signInAs(context, account);
	await page.goto(`/chat/${sessionId}`, { waitUntil: 'networkidle' });

	const thread = page.getByRole('main');
	await expect(thread.getByText('October spending')).toBeVisible();
	await expect(thread.getByRole('cell', { name: '1,250,000' })).toBeVisible();
	await expect(thread.getByText('SUM(amount)')).toBeVisible();
	await expect(thread.getByText('I can’t move money')).toBeVisible();
	await expect(thread.getByText('This reply has a chart the app can’t show yet.')).toBeVisible();

	// A decision answers with the option picked, as the next message.
	await thread.getByRole('radio', { name: /November/ }).click();
	await thread.getByRole('button', { name: /November/ }).last().click();
	await expect(thread.getByText('You answered: November')).toBeVisible();
	await expect(thread.getByText(REPLY)).toHaveCount(2);
});

test('clarifying questions are answered in the card, and the answers go as the next message', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Bikin anggaran bulanan');
	await seedReply(sessionId, {
		content: '',
		agent: 'Rena',
		blocks: [{ kind: 'questions', intro: 'Two things first', questions: [{ question: 'Which month?', options: [{ label: 'October' }, { label: 'November' }] }] }],
	});
	await signInAs(context, account);
	await page.goto(`/chat/${sessionId}`, { waitUntil: 'networkidle' });

	const thread = page.getByRole('main');
	await expect(thread.getByText('Two things first')).toBeVisible();
	await thread.getByRole('radio', { name: /October/ }).click();
	await thread.getByRole('button', { name: /Next|Review|Make the plan|Send/ }).first().click();
	await expect(thread.getByRole('paragraph').filter({ hasText: /^Which month\? October$/ })).toBeVisible();
	await expect(thread.getByRole('button', { name: /Answered 1 question/ })).toBeVisible();
	await expect(thread.getByText(REPLY)).toHaveCount(2);
	const messages = (await (await api(account, `/api/sessions/${sessionId}/messages`)).json()) as { messages: { sender: string; content: string }[] };
	expect(messages.messages.some((message) => message.sender === 'user' && message.content === 'Which month? October')).toBe(true);
});
