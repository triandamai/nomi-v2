import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { seedPendingApproval, seedPlan, seedReply } from './support/seed';

// A chat's flows on a phone against the real backend (Flow / Mobile Chat — Clarify → plan GCvfD,
// Plan → approve → act G1tK6L, Approvals & background xvntx, Errors & recovery AyrHG). The fake
// model writes only text, so the crew's questions, plans, approvals, failures and stops are put in
// the database as the crew's turn would; answering them goes through the real backend.

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

type Messages = { messages: { id: string; sender: string; content: string }[] };
const messagesOf = async (account: Account, sessionId: string) => ((await (await api(account, `/api/sessions/${sessionId}/messages`)).json()) as Messages).messages;

/** A chat whose first exchange is done (the fake model has answered), so a seeded reply is the latest. */
async function chatAfter(text: string) {
	const account = await createAccount();
	const { session_id } = (await (await api(account, '/api/sessions', { method: 'POST' })).json()) as { session_id: string };
	await api(account, `/api/sessions/${session_id}/messages`, { method: 'POST', body: JSON.stringify({ text }) });
	await expect.poll(async () => (await messagesOf(account, session_id)).some((message) => message.sender === 'assistant'), { timeout: 30_000 }).toBe(true);
	return { account, sessionId: session_id };
}

async function open(page: Page, sessionId: string) {
	await page.goto(`/chat/${sessionId}`, { waitUntil: 'networkidle' });
}

/** The crew has answered the person's last message, so what's seeded next is the latest reply. */
const answered = (account: Account, sessionId: string) =>
	expect.poll(async () => (await messagesOf(account, sessionId)).at(-1)?.sender, { timeout: 30_000 }).toBe('assistant');

const sent = async (account: Account, sessionId: string, text: string) =>
	expect.poll(async () => (await messagesOf(account, sessionId)).some((message) => message.sender === 'user' && message.content === text)).toBe(true);

test('Clarify → plan: the questions open in their sheet, park as “waiting”, and the answers go as one message', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Plan a weekend in Bandung');
	await seedReply(sessionId, {
		content: 'Two quick questions so the plan fits.',
		agent: 'Rena',
		blocks: [
			{
				kind: 'questions',
				intro: 'Two quick questions',
				questions: [
					{ question: 'How do you want to get there?', options: [{ label: 'Whoosh train', detail: '45 min' }, { label: 'Drive' }] },
					{ question: 'What should the plan cover?', multiple: true, options: [{ label: 'Train times' }, { label: 'Where to stay' }, { label: 'Food spots' }] },
				],
			},
		],
	});
	await signInAs(context, account);
	await open(page, sessionId);

	// It opens on its own, nothing picked, Next waiting (AC2, AC3).
	const sheet = page.getByRole('dialog', { name: 'Before I plan' });
	await expect(sheet).toBeVisible();
	await expect(sheet.getByText(/· 1 of 2$/)).toBeVisible();
	await expect(sheet.getByRole('button', { name: 'Next' })).toBeDisabled();

	// Your own answer: the field and Next stay in view (AC4).
	await sheet.getByRole('button', { name: 'Something else…' }).click();
	await expect(sheet.getByRole('button', { name: 'Next' })).toBeDisabled();
	await sheet.getByRole('textbox', { name: 'Your own answer' }).fill('Travel shuttle');
	await expect(sheet.getByRole('button', { name: 'Next' })).toBeInViewport();
	await sheet.getByRole('button', { name: 'Next' }).click();

	// Closing parks it; Answer comes back to the same question (AC2).
	await sheet.getByRole('button', { name: 'Close' }).click();
	await expect(page.getByRole('dialog')).toHaveCount(0);
	await page.getByRole('button', { name: 'Answer' }).click();
	await expect(page.getByRole('dialog').getByText(/· 2 of 2$/)).toBeVisible();

	// Many of many counts its ticks (AC5).
	const again = page.getByRole('dialog');
	await expect(again.getByRole('button', { name: 'Review 0' })).toBeDisabled();
	await again.getByRole('checkbox', { name: /Train times/ }).check();
	await again.getByRole('checkbox', { name: /Where to stay/ }).check();
	await again.getByRole('button', { name: 'Review 2' }).click();
	const review = page.getByRole('dialog', { name: 'Your answers' });
	await expect(review.getByText('Nothing runs yet')).toBeVisible();
	await review.getByRole('button', { name: 'Make the plan' }).click();

	// One message with every answer, and the thread folds them (AC6).
	await sent(account, sessionId, 'How do you want to get there? Travel shuttle\nWhat should the plan cover? Train times, Where to stay');
	await expect(page.getByRole('button', { name: /Answered 2 questions/ })).toBeVisible();
});

test('Clarify → plan: a proposed plan shows its bar; Run plan sends the choice, Keep planning puts the note over the composer', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Plan a weekend in Bandung');
	const first = '- [ ] Find Whoosh trains\n- [ ] Food spots\n- [ ] Share with Sari';
	const second = '- [ ] Find Whoosh trains\n- [ ] Rainy-day backup\n- [ ] Share with Sari';
	const { agentSessionId, planId } = await seedPlan(sessionId, account.email, { title: '1–2 Nov', versions: [first, second] });
	await seedReply(sessionId, { content: 'Food spots removed, rainy-day backup added.', agent: 'Rena', blocks: [{ kind: 'plan', plan_id: planId, agent_session_id: agentSessionId, title: '1–2 Nov' }] });
	await signInAs(context, account);
	await open(page, sessionId);

	// The revision: V2, the added step marked + and nothing to reorder or remove (AC8, AC9).
	await expect(page.getByRole('heading', { name: 'Draft plan · 1–2 Nov' })).toBeVisible();
	await expect(page.getByText('V2', { exact: true })).toBeVisible();
	await expect(page.getByRole('listitem').filter({ hasText: 'Rainy-day backup' })).toContainText('+');
	await expect(page.getByRole('listitem').filter({ hasText: 'Find Whoosh trains' })).not.toContainText('+');
	await expect(page.getByText(/Reorder|reassign|~\d+ min/)).toHaveCount(0);
	await expect(page.getByText('Tell Nomi what to change. Nothing runs until you say so.')).toBeVisible();

	// Keep planning: the note over the composer, the bar goes (AC7).
	await expect(page.getByText('Plan v2 ready')).toBeVisible();
	await page.getByRole('button', { name: 'Review & run' }).click();
	const run = page.getByRole('dialog', { name: 'Run this plan?' });
	await expect(run.getByRole('radio', { name: /Run it, ask before anything leaves Nomi/ })).toBeChecked();
	await run.getByText('Keep planning').click();
	await run.getByRole('button', { name: 'Run plan' }).click();
	await expect(page.getByText('Changing the plan · v2')).toBeVisible();
	await expect(page.getByRole('textbox', { name: /message/i })).toBeFocused();
	await expect(page.getByText('Plan v2 ready')).toHaveCount(0);

	// Run plan sends the chosen way as the person's message (D1).
	await page.reload({ waitUntil: 'networkidle' });
	await page.getByRole('button', { name: 'Review & run' }).click();
	await page.getByRole('dialog', { name: 'Run this plan?' }).getByRole('button', { name: 'Run plan' }).click();
	await sent(account, sessionId, 'Run it, ask before anything leaves Nomi');
});

test('Clarify → plan: a first plan’s bar opens the run sheet; Research only says nothing is shared; a form that isn’t a reminder has its own card', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Plan dinner on Friday');
	const { agentSessionId, planId } = await seedPlan(sessionId, account.email, { title: 'Friday dinner', versions: ['- [ ] Find a table for 4\n- [ ] Book it\n- [ ] Tell Sari'] });
	await seedReply(sessionId, { content: 'Here’s the plan.', agent: 'Rena', blocks: [{ kind: 'plan', plan_id: planId, agent_session_id: agentSessionId, title: 'Friday dinner' }] });
	await signInAs(context, account);
	await open(page, sessionId);

	// One bar for a plan nothing runs yet: it opens the sheet, it runs nothing (gate 3 #1).
	await expect(page.getByText('Plan ready · nothing runs yet')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Run plan' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Review & run' }).click();
	const run = page.getByRole('dialog', { name: 'Run this plan?' });
	await expect(run.getByText('3 steps · Rena')).toBeVisible();
	await expect(run.getByRole('radio', { name: /Research only.*Nothing shared\./ })).toBeVisible();
	await run.getByRole('button', { name: 'Not now' }).click();
	await expect(page.getByText('Plan ready · nothing runs yet')).toBeVisible();

	// A form that isn't a reminder: its own card, its date as the content rules write it.
	await page.getByRole('textbox', { name: /message/i }).fill('Book it');
	await page.getByRole('button', { name: 'Send' }).click();
	await answered(account, sessionId);
	await seedReply(sessionId, {
		content: '',
		agent: 'Nomi',
		blocks: [{ kind: 'view', view: 'form', title: 'Dinner booking', fields: [{ key: 'date', label: 'Date', type: 'date', required: true, default: '2030-11-29' }, { key: 'people', label: 'People', type: 'number', default: 4 }] }],
	});
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByText('29 Nov 2030', { exact: false })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Edit reminder' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Edit Dinner booking' }).click();
	await expect(page.getByRole('dialog', { name: 'Dinner booking' })).toBeVisible();
});

test('Plan → approve → act: a decision, a form whose date has passed, and an approval that’s decided', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Track my kos rent');
	await seedReply(sessionId, {
		content: 'When should I remind you about rent?',
		agent: 'Kala',
		blocks: [{ kind: 'view', view: 'decision', question: 'When should I remind you?', options: [{ label: 'On the 25th' }, { label: '3 days before', recommended: true }] }],
	});
	await signInAs(context, account);
	await open(page, sessionId);

	// The suggested option first, picked; “Use {option}” answers with it.
	await page.getByRole('button', { name: 'Use 3 days before' }).click();
	await sent(account, sessionId, '3 days before');
	await answered(account, sessionId);

	// The form: its summary card opens the sheet; a past date holds Save (AC11).
	await seedReply(sessionId, {
		content: '',
		agent: 'Kala',
		blocks: [
			{
				kind: 'view',
				view: 'form',
				title: 'New reminder',
				fields: [
					{ key: 'what', label: 'What', type: 'text', required: true, default: 'Pay kos rent to Bu Sri' },
					{ key: 'date', label: 'Date', type: 'date', required: true, default: '2026-01-25' },
					{ key: 'repeat', label: 'Repeat', type: 'select', options: ['Once', 'Weekly', 'Monthly', 'Yearly'], default: 'Monthly' },
				],
			},
		],
	});
	await page.reload({ waitUntil: 'networkidle' });
	await page.getByRole('button', { name: 'Edit reminder' }).click();
	const form = page.getByRole('dialog', { name: 'New reminder' });
	await expect(form.getByText('Kala · prefilled from chat')).toBeVisible();
	await expect(form.getByText('That date has passed.')).toBeVisible();
	await expect(form.getByRole('button', { name: 'Save' })).toHaveAttribute('aria-disabled', 'true');
	await form.getByLabel('Date').fill('2030-11-25');
	await form.getByRole('button', { name: 'Save' }).click();
	await sent(account, sessionId, 'New reminder\nWhat: Pay kos rent to Bu Sri\nDate: 2030-11-25\nRepeat: Monthly');
	await answered(account, sessionId);

	// An approval to send an email: Don't send (D5), its full preview without Edit or the switch
	// (AC13), and Approve recorded once, folding the card (AC12).
	const approvalId = await seedReply(sessionId, {
		content: '',
		agent: 'Tara',
		blocks: [
			{
				kind: 'approval_request',
				id: crypto.randomUUID(),
				tool_name: 'gmail.send',
				description: 'Tara wants to send an email',
				input: { to: 'bu.sri@example.com', subject: 'Transfer sewa November', body: 'Selamat siang Bu Sri, sewa sudah saya transfer.' },
				status: 'pending',
				decided_at: null,
			},
		],
	});
	await seedPendingApproval(sessionId, account.email, approvalId);
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByText('Tara · needs you', { exact: true })).toBeVisible();
	const card = page.getByRole('region', { name: 'Tara needs your OK' });
	await expect(card.getByRole('button', { name: 'Don’t send' })).toBeVisible();
	await card.getByRole('button', { name: /Tara wants to send an email/ }).click();
	const preview = page.getByRole('dialog', { name: 'Send this email?' });
	await expect(preview.getByText('Tara · via your Gmail')).toBeVisible();
	await expect(preview.getByText('bu.sri@example.com')).toBeVisible();
	await expect(preview.getByRole('button', { name: 'Edit' })).toHaveCount(0);
	await expect(preview.getByRole('switch')).toHaveCount(0);
	const decided = page.waitForResponse((response) => response.url().includes(`/messages/${approvalId}/approval`));
	await preview.getByRole('button', { name: 'Approve & send' }).click();
	expect((await decided).ok()).toBe(true);
	await expect(page.getByText('Tara sent the email to bu.sri@example.com')).toBeVisible();
	await expect(page.getByText(/^Approved by you · sent \d{2}:\d{2}$/)).toBeVisible();
});

test('Approvals: an approval no longer pending reloads and stays as the backend has it (409)', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Email Bu Sri');
	await seedReply(sessionId, {
		content: '',
		agent: 'Tara',
		blocks: [{ kind: 'approval_request', id: crypto.randomUUID(), tool_name: 'files.delete', description: 'Tara wants to delete a file', input: { path: 'old.txt' }, status: 'pending', decided_at: null }],
	});
	await signInAs(context, account);
	await open(page, sessionId);
	const card = page.getByRole('region', { name: 'Tara needs your OK' });
	await expect(card.getByRole('button', { name: 'Decline' })).toBeVisible();
	const decided = page.waitForResponse((response) => response.url().includes('/approval'));
	await card.getByRole('button', { name: 'Approve' }).click();
	expect((await decided).status()).toBe(409);
	await expect(card.getByRole('button', { name: 'Approve' })).toBeVisible();
});

test('Approvals & background: quick replies and the Supervisor’s report, with Resume', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Send the itinerary');
	await seedReply(sessionId, {
		content: 'Okay, nothing was sent.',
		agent: 'Tara',
		blocks: [{ kind: 'view', view: 'quick_replies', replies: ['Make it more formal', 'Drop it'] }],
	});
	await signInAs(context, account);
	await open(page, sessionId);
	await page.getByRole('button', { name: 'Drop it' }).click();
	await sent(account, sessionId, 'Drop it');

	await answered(account, sessionId);
	await seedReply(sessionId, {
		content: '',
		agent: 'Supervisor',
		blocks: [
			{
				kind: 'stopped',
				tasks: [
					{ agent: 'Nomi', title: 'Find Whoosh trains', state: 'done' },
					{ agent: 'Nomi', title: 'Shortlist guesthouses', detail: '2 of 3', state: 'stopped' },
					{ agent: 'Tara', title: 'Share with Sari', state: 'waiting' },
				],
			},
		],
	});
	await page.reload({ waitUntil: 'networkidle' });
	await expect(page.getByText('Supervisor stopped 3 tasks')).toBeVisible();
	for (const text of ['stopped · 2 of 3', 'not started', 'Nothing sent was undone.']) await expect(page.getByText(text, { exact: true })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Discard' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Resume' }).click();
	await sent(account, sessionId, 'Pick up where you left off');
});

test('Errors & recovery: a failed turn is the phone’s error card, with the desktop’s moves and no provider quote', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('What did I spend?');
	await seedReply(sessionId, {
		content: 'I couldn’t reply: the AI provider rejected the API key.',
		agent: 'Nomi',
		blocks: [{ kind: 'run_failed', retry: false }],
	});
	await signInAs(context, account);
	await open(page, sessionId);
	const card = page.getByRole('alert');
	await expect(card).toContainText('Key rejected');
	await expect(card.getByRole('link', { name: 'Update key' })).toHaveAttribute('href', '/settings/models');
	await expect(page.getByText(/ said$/i)).toHaveCount(0);
});

test('Errors & recovery: offline the message waits with Cancel, and goes once back', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Hello');
	await signInAs(context, account);
	await open(page, sessionId);
	await context.setOffline(true);
	await page.evaluate(() => window.dispatchEvent(new Event('offline')));
	await expect(page.getByText('You’re offline. Messages send when you’re back.')).toBeVisible();
	await page.getByRole('textbox', { name: /message/i }).fill('Food spending in October?');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText('queued · sends when you’re back')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
	await context.setOffline(false);
	await page.evaluate(() => window.dispatchEvent(new Event('online')));
	await expect(page.getByText('Back online. 1 queued message sent.')).toBeVisible();
	await sent(account, sessionId, 'Food spending in October?');
});

test('Errors & recovery: signed out mid-chat, a sheet that can’t be dismissed keeps the draft', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Hello');
	await signInAs(context, account);
	await open(page, sessionId);
	await context.clearCookies();
	await page.getByRole('textbox', { name: /message/i }).fill('Find Whoosh trains for Sat 1 Nov');
	await page.getByRole('button', { name: 'Send' }).click();
	const sheet = page.getByRole('alertdialog', { name: /signed out/ });
	await expect(sheet).toBeVisible();
	await expect(sheet.getByText('kept')).toBeVisible();
	await expect(sheet.getByText('Find Whoosh trains for Sat 1 Nov')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(sheet).toBeVisible();
	await expect(sheet.getByRole('link', { name: 'Sign in again' })).toBeVisible();
});

test('Approvals & background: elsewhere in the app, a new approval in a chat is said with a toast that opens it', async ({ page, context }) => {
	const { account, sessionId } = await chatAfter('Email Bu Sri about the rent');
	expect((await api(account, `/api/sessions/${sessionId}`, { method: 'PATCH', body: JSON.stringify({ title: 'Kos rent tracker' }) })).ok).toBe(true);
	await signInAs(context, account);
	await page.goto('/chats', { waitUntil: 'networkidle' });

	const approvalId = await seedReply(sessionId, {
		content: '',
		agent: 'Tara',
		blocks: [{ kind: 'approval_request', id: crypto.randomUUID(), tool_name: 'gmail.send', description: 'Send an email', input: { to: 'bu.sri@example.com' }, status: 'pending', decided_at: null }],
	});
	await seedPendingApproval(sessionId, account.email, approvalId);
	// The chat moves on (as the crew's turn would announce it): the person's feed hears of it.
	await api(account, `/api/sessions/${sessionId}/messages`, { method: 'POST', body: JSON.stringify({ text: 'Any news?' }) });

	const toast = page.getByRole('status').filter({ hasText: 'Kos rent tracker: Tara needs your OK' });
	await expect(toast).toBeVisible({ timeout: 30_000 });
	await toast.getByRole('button', { name: 'Open' }).click();
	await expect(page).toHaveURL(new RegExp(`/chat/${sessionId}$`));
});
