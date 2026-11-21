import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { RotateCcw } from '@lucide/svelte';
import PhoneClarifySheet from './PhoneClarifySheet.svelte';
import PhoneFlowBar from './PhoneFlowBar.svelte';
import PhoneRunSheet from './PhoneRunSheet.svelte';
import PhoneDraftPlan from './PhoneDraftPlan.svelte';
import PhoneDecision from './PhoneDecision.svelte';
import PhoneFormSheet from './PhoneFormSheet.svelte';
import PhoneApprovalSheet from './PhoneApprovalSheet.svelte';
import PhoneQuickReplies from './PhoneQuickReplies.svelte';
import PhoneCrewWorking from './PhoneCrewWorking.svelte';
import PhoneSupervisor from './PhoneSupervisor.svelte';
import PhoneErrorCard from './PhoneErrorCard.svelte';
import PhoneCutNotice from './PhoneCutNotice.svelte';
import PhonePendingMessage from './PhonePendingMessage.svelte';
import PhoneSignedOutSheet from './PhoneSignedOutSheet.svelte';
import PhoneComposer from './PhoneComposer.svelte';
import PhoneFormCard from './PhoneFormCard.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The phone's chat flows (Flow / Mobile Chat — Clarify → plan, Plan → approve → act, Approvals &
// background, Errors & recovery): the pieces the phone chat puts together.

afterEach(() => {
	document.body.innerHTML = '';
});
const box = (element: Element) => element.getBoundingClientRect();

const questions = [
	{ topic: 'Question 1', title: 'How do you want to get there?', options: [{ title: 'Whoosh train', detail: '45 min' }, { title: 'Drive' }, { title: 'Travel shuttle' }] },
	{ topic: 'Question 2', title: 'Which weekend?', options: [{ title: '1–2 Nov', recommended: true }, { title: '8–9 Nov' }] },
	{ topic: 'Question 3', title: 'What should the plan cover?', multiple: true, options: [{ title: 'Train times' }, { title: 'Where to stay' }, { title: 'Food spots' }] },
];

describe('PhoneClarifySheet', () => {
	it('asks one question at a time: nothing picked and Next waits, until a pick', async () => {
		await page.viewport(390, 844);
		await render(PhoneClarifySheet, { open: true, chat: 'Bandung weekend', questions, onfinish: () => {} });
		const sheet = page.getByRole('dialog', { name: 'Before I plan' });
		await expect.element(sheet.getByText('Bandung weekend · 1 of 3')).toBeVisible();
		await expect.element(sheet.getByRole('heading', { name: 'How do you want to get there?' })).toBeVisible();
		expect(getComputedStyle(sheet.getByRole('heading', { name: 'How do you want to get there?' }).element()).fontSize).toBe('20px');
		expect(sheet.getByRole('radio').elements().every((radio) => !(radio as HTMLInputElement).checked)).toBe(true);
		await expect.element(sheet.getByRole('button', { name: 'Next' })).toBeDisabled();
		await sheet.getByText('Drive').click();
		await expect.element(sheet.getByRole('button', { name: 'Next' })).toBeEnabled();
		await sheet.getByRole('button', { name: 'Next' }).click();
		await expect.element(sheet.getByText('Bandung weekend · 2 of 3')).toBeVisible();
		// A recommended option starts picked, tagged.
		await expect.element(sheet.getByRole('radio', { name: /1–2 Nov/ })).toBeChecked();
		await expect.element(sheet.getByText('Recommended')).toBeVisible();
	});

	it('takes the person’s own answer: the options fold away, Next waits for text', async () => {
		await render(PhoneClarifySheet, { open: true, chat: 'Bandung weekend', questions, onfinish: () => {} });
		const sheet = page.getByRole('dialog');
		await sheet.getByRole('button', { name: 'Something else…' }).click();
		const field = sheet.getByRole('textbox', { name: 'Your own answer' });
		await expect.element(field).toHaveFocus();
		await expect.element(sheet.getByRole('radio')).not.toBeInTheDocument();
		await expect.element(sheet.getByRole('button', { name: 'Next' })).toBeDisabled();
		await expect.element(sheet.getByRole('button', { name: 'Back' })).toBeVisible();
		await field.fill('Rp 3,5 jt max');
		await expect.element(sheet.getByRole('button', { name: 'Next' })).toBeEnabled();
		await sheet.getByRole('button', { name: 'Show the 3 suggestions' }).click();
		await expect.element(sheet.getByRole('radio').first()).toHaveFocus();
	});

	it('ticks many of many and counts them; Skip takes the recommendation as a default; the review goes back', async () => {
		const onfinish = vi.fn();
		await render(PhoneClarifySheet, { open: true, chat: 'Bandung weekend', questions, onfinish });
		const sheet = page.getByRole('dialog');
		await sheet.getByText('Whoosh train').click();
		await sheet.getByRole('button', { name: 'Next' }).click();
		await sheet.getByRole('button', { name: 'Skip' }).click();
		// Answered segments are accent, the current one text, the rest border-strong (L42Ej, U8a64).
		const segments = [...document.querySelectorAll('[data-segment]')].map((one) => getComputedStyle(one).backgroundColor);
		expect(segments).toEqual([tokenBackground('nm-accent'), tokenBackground('nm-accent'), tokenBackground('nm-text')]);
		await expect.element(sheet.getByRole('button', { name: 'Review 0' })).toBeDisabled();
		// Many of many: plain rows 6 apart, a 22 box (r6) that fills with text once ticked, never accent.
		const rows = sheet.getByRole('checkbox').elements().map((box) => box.closest('label')!);
		expect(box(rows[1]).top - box(rows[0]).bottom).toBe(6);
		expect(getComputedStyle(rows[0]).backgroundColor).not.toBe(tokenBackground('nm-accent-bg'));
		// The native box is visually hidden behind the drawn one: the row (its label) is what's tapped.
		await sheet.getByText('Train times').click();
		await sheet.getByText('Food spots').click();
		const ticked = rows[0].querySelector('span[aria-hidden="true"]:last-child')!;
		expect([box(ticked).width, getComputedStyle(ticked).borderRadius, getComputedStyle(ticked).backgroundColor]).toEqual([22, '6px', tokenBackground('nm-text')]);
		expect(getComputedStyle(rows[0]).borderColor).toBe(tokenColor('nm-border-strong'));
		await sheet.getByRole('button', { name: 'Review 2' }).click();

		const review = page.getByRole('dialog', { name: 'Your answers' });
		await expect.element(review.getByText('Nothing runs yet')).toBeVisible();
		await expect.element(review.getByRole('button', { name: 'Change Which weekend?: 1–2 Nov' })).toBeVisible();
		await expect.element(review.getByText('Default', { exact: true })).toBeVisible();
		await expect.element(review.getByText('You skipped question 2, so I picked 1–2 Nov.')).toBeVisible();
		await review.getByRole('button', { name: 'Change How do you want to get there?: Whoosh train' }).click();
		await expect.element(page.getByText('Bandung weekend · 1 of 3')).toBeVisible();
		await expect.element(page.getByRole('radio', { name: /Whoosh train/ })).toBeChecked();
		for (let step = 0; step < 3; step += 1) await page.getByRole('button', { name: /^(Next|Review 2)$/ }).click();
		await page.getByRole('button', { name: 'Make the plan' }).click();
		expect(onfinish).toHaveBeenCalledWith({ step: 3, answers: [{ choice: 0 }, { choice: 0 }, { choices: [0, 2] }], skipped: [false, true, false] });
	});

	it('a skip with nothing recommended reads Skipped, with no note', async () => {
		await render(PhoneClarifySheet, { open: true, chat: 'Trip', questions: [questions[0]], onfinish: () => {} });
		await page.getByRole('button', { name: 'Skip' }).click();
		await expect.element(page.getByRole('button', { name: 'Change How do you want to get there?: Skipped' })).toBeVisible();
		await expect.element(page.getByText(/You skipped/)).not.toBeInTheDocument();
	});
});

describe('PhoneFlowBar', () => {
	it('says what waits, with its one action as a mint pill 36 tall and a 44 target', async () => {
		await page.viewport(390, 844);
		const onclick = vi.fn();
		await render(PhoneFlowBar, { label: 'Plan ready · nothing runs yet', action: 'Review & run', onclick });
		const button = page.getByRole('button', { name: 'Review & run' });
		expect(box(button.element()).height).toBe(36);
		expect(getComputedStyle(button.element()).backgroundColor).toBe(tokenBackground('nm-accent'));
		const target = button.element().querySelector('span')!;
		expect(box(target).height).toBe(44);
		expect(box(document.querySelector('[data-flow-bar]')!).height).toBe(56);
		await button.click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('PhoneRunSheet', () => {
	it('starts on the recommended way, and Run plan gives the one chosen', async () => {
		const onrun = vi.fn();
		await render(PhoneRunSheet, {
			open: true,
			subtitle: '5 steps · Nomi, Dana, Tara',
			options: [{ title: 'Run it, ask before anything leaves Nomi', recommended: true }, { title: 'Research only' }, { title: 'Keep planning', detail: 'Tell Nomi what to change' }],
			onrun,
		});
		const sheet = page.getByRole('dialog', { name: 'Run this plan?' });
		await expect.element(sheet.getByText('5 steps · Nomi, Dana, Tara')).toBeVisible();
		await expect.element(sheet.getByRole('radio', { name: /Run it/ })).toBeChecked();
		await sheet.getByText('Keep planning').click();
		await sheet.getByRole('button', { name: 'Run plan' }).click();
		expect(onrun).toHaveBeenCalledWith(2);
	});
});

describe('PhoneDraftPlan', () => {
	it('draws a draft plan with no way to reorder, reassign or remove, and a revision’s new steps marked +', async () => {
		// `intro` is also a mount option, so the props go in `props`.
		await render(PhoneDraftPlan, {
			props: {
				title: '1–2 Nov',
				version: 2,
				intro: 'Tell Nomi what to change. Nothing runs until you say so.',
				steps: [
					{ label: 'Find Whoosh trains', owner: { agent: 'nomi', name: 'Nomi' } },
					{ label: 'Rainy-day backup', owner: { agent: 'nomi', name: 'Nomi' }, added: true },
				],
			},
		});
		await expect.element(page.getByRole('heading', { name: 'Draft plan · 1–2 Nov' })).toBeVisible();
		await expect.element(page.getByText('V2')).toBeVisible();
		await expect.element(page.getByRole('button')).not.toBeInTheDocument();
		const added = page.getByRole('listitem').nth(1).element();
		expect(getComputedStyle(added).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		expect(added.textContent).toContain('+');
		await expect.element(page.getByText('01')).toBeVisible();
	});

	it('gives a long step the row’s width, its owner under it, on a phone (issue #334)', async () => {
		await page.viewport(390, 844);
		const label = 'Build database tables and customer booking frontend with QRIS upload';
		await render(PhoneDraftPlan, { props: { title: 'Barberfi', intro: 'x', steps: [{ label, owner: { agent: 'koda', name: 'Koda' }, added: true }] } });
		const text = page.getByText(label).element().getBoundingClientRect();
		const chip = page.getByText('Koda', { exact: true }).element().getBoundingClientRect();
		expect(chip.top).toBeGreaterThanOrEqual(text.bottom);
		const row = page.getByRole('listitem').element().getBoundingClientRect();
		expect(text.left - row.left).toBeGreaterThanOrEqual(8);
	});

	it('says DRAFT on the first version', async () => {
		await render(PhoneDraftPlan, { props: { title: 'Rent', intro: 'x', steps: [{ label: 'Read transfers', owner: { agent: 'maya', name: 'Maya' } }] } });
		await expect.element(page.getByText('Draft', { exact: true })).toBeVisible();
	});
});

describe('PhoneDecision', () => {
	it('puts the suggested option first and picked, and answers with “Use {option}”', async () => {
		const onconfirm = vi.fn();
		await render(PhoneDecision, { question: 'When should I remind you?', options: [{ title: 'On the day' }, { title: '3 days before', suggested: true }], onconfirm });
		await expect.element(page.getByRole('radio').first()).toHaveAccessibleName(/3 days before/);
		await page.getByRole('button', { name: 'Use 3 days before' }).click();
		expect(onconfirm).toHaveBeenCalledWith(1);
	});
});

describe('PhoneFormSheet', () => {
	it('says when a date has passed, and Save waits until it’s fixed', async () => {
		const onsubmit = vi.fn();
		await render(PhoneFormSheet, {
			open: true,
			title: 'New reminder',
			subtitle: 'Kala · prefilled from chat',
			today: '2026-11-17',
			fields: [
				{ key: 'what', label: 'What', type: 'text', required: true, options: [] },
				{ key: 'date', label: 'Date', type: 'date', required: true, options: [] },
				{ key: 'repeat', label: 'Repeat', type: 'select', required: false, options: ['Once', 'Weekly', 'Monthly', 'Yearly'] },
			],
			values: { what: 'Pay kos rent', date: '2026-10-25', repeat: 'Monthly' },
			onsubmit,
		});
		const sheet = page.getByRole('dialog', { name: 'New reminder' });
		await expect.element(sheet.getByText('That date has passed.')).toBeVisible();
		await expect.element(sheet.getByLabelText('Date')).toHaveAttribute('aria-invalid', 'true');
		expect(getComputedStyle(sheet.getByLabelText('Date').element()).fontFamily).toMatch(/Geist Mono/);
		expect(getComputedStyle(sheet.getByLabelText('What').element()).fontFamily).not.toMatch(/Mono/);
		await expect.element(sheet.getByRole('button', { name: 'Save' })).toHaveAttribute('aria-disabled', 'true');
		sheet.getByRole('button', { name: 'Save' }).element().dispatchEvent(new MouseEvent('click', { bubbles: true }));
		expect(onsubmit).not.toHaveBeenCalled();
		await expect.element(sheet.getByRole('radio', { name: 'Monthly' })).toHaveAttribute('aria-checked', 'true');
		expect(box(sheet.getByRole('radio', { name: 'Monthly' }).element().querySelector('span')!).height).toBe(44);
		await sheet.getByLabelText('Date').fill('2026-11-25');
		await expect.element(sheet.getByText('That date has passed.')).not.toBeInTheDocument();
		await sheet.getByRole('button', { name: 'Save' }).click();
		expect(onsubmit).toHaveBeenCalledWith({ what: 'Pay kos rent', date: '2026-11-25', repeat: 'Monthly' });
	});
});

describe('PhoneApprovalSheet', () => {
	it('shows everything that goes out, with no Edit and no “Don’t ask again”', async () => {
		const onapprove = vi.fn();
		await render(PhoneApprovalSheet, {
			open: true,
			title: 'Send this email?',
			subtitle: 'Tara · via your Gmail',
			fields: [
				{ key: 'To', value: 'bu.sri@example.com' },
				{ key: 'Subject', value: 'Transfer sewa November' },
			],
			body: 'Selamat siang Bu Sri',
			attachment: 'bukti-transfer-nov.jpg',
			approveLabel: 'Approve & send',
			onapprove,
		});
		const sheet = page.getByRole('dialog', { name: 'Send this email?' });
		await expect.element(sheet.getByText('bu.sri@example.com')).toBeVisible();
		await expect.element(sheet.getByRole('switch')).not.toBeInTheDocument();
		await expect.element(sheet.getByRole('button', { name: 'Edit' })).not.toBeInTheDocument();
		await sheet.getByRole('button', { name: 'Approve & send' }).click();
		expect(onapprove).toHaveBeenCalledOnce();
	});
});

describe('the thread’s phone pieces', () => {
	it('PhoneQuickReplies: full-width pills, 44 tall', async () => {
		await page.viewport(390, 844);
		const onpick = vi.fn();
		await render(PhoneQuickReplies, { replies: ['Make it more formal', 'Drop it'], onpick });
		const pill = page.getByRole('button', { name: 'Drop it' });
		expect(box(pill.element()).height).toBe(44);
		await pill.click();
		expect(onpick).toHaveBeenCalledWith('Drop it');
	});

	it('PhoneCrewWorking: how many work, and Stop all', async () => {
		const onstop = vi.fn();
		await render(PhoneCrewWorking, { crew: ['nomi', 'dana', 'tara'], onstop });
		await expect.element(page.getByRole('status')).toHaveTextContent('3 crew working');
		await page.getByRole('button', { name: 'Stop all' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('PhoneSupervisor: each task’s real state, Nothing sent was undone, Resume and no Discard', async () => {
		const onresume = vi.fn();
		await render(PhoneSupervisor, {
			tasks: [
				{ agent: 'nomi', title: 'Find Whoosh trains', state: 'done' },
				{ agent: 'nomi', title: 'Shortlist guesthouses', detail: '2 of 3', state: 'stopped' },
				{ agent: 'tara', title: 'Share with Sari', state: 'waiting' },
			],
			onresume,
		});
		await expect.element(page.getByText('Supervisor stopped 3 tasks')).toBeVisible();
		for (const text of ['done', 'stopped · 2 of 3', 'not started', 'Nothing sent was undone.']) await expect.element(page.getByText(text, { exact: true })).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Discard' })).not.toBeInTheDocument();
		// Resume keeps the pen's 30 (with a 44 target); the footer is 50 (nQiZd).
		const resume = page.getByRole('button', { name: 'Resume' }).element();
		expect([box(resume).height, box(resume.querySelector('span')!).height, box(resume.closest('footer')!).height]).toEqual([30, 44, 50]);
		await page.getByRole('button', { name: 'Resume' }).click();
		expect(onresume).toHaveBeenCalledOnce();
	});

	it('PhoneErrorCard: the sentence, the title on danger-bg, the model; no code and no “said” quote', async () => {
		const onretry = vi.fn();
		await render(PhoneErrorCard, {
			title: 'Out of credits',
			sentence: 'I couldn’t reply: your DeepSeek account is out of credits.',
			meta: 'deepseek-flash',
			primary: { label: 'Switch to Nomi credits' },
			secondary: { label: 'Retry', icon: RotateCcw, onclick: onretry },
		});
		const card = page.getByRole('alert');
		await expect.element(card).toHaveAccessibleName('Out of credits. I couldn’t reply: your DeepSeek account is out of credits.');
		expect(getComputedStyle(card.element().querySelector('header')!).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		await expect.element(page.getByText(/said/i)).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Retry' }).click();
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('PhoneCutNotice: when it was cut off, Start over and Continue', async () => {
		const oncontinue = vi.fn();
		await render(PhoneCutNotice, { seconds: 18, oncontinue, onrestart: () => {} });
		await expect.element(page.getByText('Reply cut off after 18s')).toBeVisible();
		expect(getComputedStyle(page.getByRole('status').element()).backgroundColor).toBe(tokenBackground('nm-warning-bg'));
		await page.getByRole('button', { name: 'Continue' }).click();
		expect(oncontinue).toHaveBeenCalledOnce();
	});

	it('PhonePendingMessage: queued with Cancel, or not sent with the reason in danger', async () => {
		const oncancel = vi.fn();
		await render(PhonePendingMessage, { text: 'How much on food?', status: 'queued', note: 'queued · sends when you’re back', action: { label: 'Cancel', onclick: oncancel } });
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(oncancel).toHaveBeenCalledOnce();
		document.body.innerHTML = '';
		await render(PhonePendingMessage, { text: 'Read the statements', status: 'failed', note: 'not sent · Nomi server error 500', action: { label: 'Retry', onclick: () => {} } });
		expect(getComputedStyle(page.getByText('not sent · Nomi server error 500').element()).color).toBe(tokenColor('nm-danger'));
	});

	it('PhoneSignedOutSheet: can’t be dismissed, keeps the draft, focuses Sign in again', async () => {
		await render(PhoneSignedOutSheet, { open: true, draft: 'Find Whoosh trains for Sat 1 Nov…', signInHref: '/sign-in?next=/chat/1' });
		const sheet = page.getByRole('alertdialog', { name: /signed out/ });
		await expect.element(sheet.getByText('kept')).toBeVisible();
		await expect.element(sheet.getByRole('link', { name: 'Sign in again' })).toHaveFocus();
		await expect.element(sheet.getByRole('button', { name: 'Close' })).not.toBeInTheDocument();
		await userEvent.keyboard('{Escape}');
		await expect.element(sheet).toBeVisible();
	});

	it('PhoneComposer: says what the next message is for over the field', async () => {
		await render(PhoneComposer, { placeholder: 'Message Nomi', notice: 'Changing the plan · v1', onsend: () => {} });
		await expect.element(page.getByText('Changing the plan · v1')).toBeVisible();
	});
});

describe('PhoneFormCard', () => {
	it('folds a form that isn’t a reminder: a neutral tile, what’s filled in mono, and “Edit {title}”', async () => {
		const onedit = vi.fn();
		await render(PhoneFormCard, { title: 'Dinner booking', values: '28 Nov 2026 · 4 people', owner: { agent: 'nomi', name: 'Nomi' }, onedit });
		expect(getComputedStyle(page.getByText('28 Nov 2026 · 4 people').element()).fontFamily).toMatch(/Geist Mono/);
		await expect.element(page.getByRole('button', { name: 'Edit reminder' })).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Edit Dinner booking' }).click();
		expect(onedit).toHaveBeenCalledOnce();
	});
});
