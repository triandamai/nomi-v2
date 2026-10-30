import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Send } from '@lucide/svelte';
import ApprovalCard from './ApprovalCard.svelte';
import DecisionCard from './DecisionCard.svelte';
import SelectionCard from './SelectionCard.svelte';
import FormCardFixture from './FormCardFixture.svelte';
import FollowUps from './FollowUps.svelte';
import QuickReplies from './QuickReplies.svelte';
import ClarifyCard from './ClarifyCard.svelte';
import ClarifyReview from './ClarifyReview.svelte';
import ClarifyAnswered from './ClarifyAnswered.svelte';
import PlanApproval from './PlanApproval.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Actions: anything that sends, spends, deletes or decides for you stops here and
// waits. One clear primary per card; the outcome stays visible after you act.

describe('ApprovalCard', () => {
	const pending = {
		agent: 'tara' as const,
		title: 'Tara wants to send an email',
		fields: [
			{ key: 'to', value: 'bu.sri.kosmawar@gmail.com' },
			{ key: 'subject', value: 'Transfer sewa November sudah masuk' },
		],
		body: 'Selamat siang Bu Sri, sewa kamar 3 untuk November sudah saya transfer.',
		attachment: 'bukti-transfer-nov.jpg · 212 KB',
		rememberLabel: "Don't ask again for Bu Sri",
		approveLabel: 'Approve & send',
		approveIcon: Send,
		rejectLabel: "Don't send",
	};

	it('shows exactly what will happen and waits for you', async () => {
		const onapprove = vi.fn();
		const onreject = vi.fn();
		const onedit = vi.fn();
		await render(ApprovalCard, { ...pending, onapprove, onreject, onedit });
		await expect.element(page.getByText('Needs you')).toBeVisible();
		expect(getComputedStyle(page.getByText('to', { exact: true }).element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByText('bu.sri.kosmawar@gmail.com')).toBeVisible();
		await expect.element(page.getByText('bukti-transfer-nov.jpg · 212 KB')).toBeVisible();
		await page.getByRole('button', { name: 'Edit' }).click();
		await page.getByRole('button', { name: "Don't send" }).click();
		await page.getByRole('button', { name: 'Approve & send' }).click();
		expect(onedit).toHaveBeenCalledOnce();
		expect(onreject).toHaveBeenCalledOnce();
		expect(onapprove).toHaveBeenCalledWith({ remember: false });
	});

	it('remembers the choice for next time only when you tick it', async () => {
		const onapprove = vi.fn();
		await render(ApprovalCard, { ...pending, onapprove });
		await page.getByRole('checkbox', { name: "Don't ask again for Bu Sri" }).click();
		await page.getByRole('button', { name: 'Approve & send' }).click();
		expect(onapprove).toHaveBeenCalledWith({ remember: true });
	});

	it('collapses to the outcome once approved, with where to see it', async () => {
		const { container } = await render(ApprovalCard, {
			agent: 'tara',
			title: 'Tara sent the email to Bu Sri',
			status: 'approved',
			outcome: 'Approved by you · sent 14:06',
			action: { label: 'View in Gmail', href: 'https://mail.google.com' },
		});
		expect(container.querySelector('svg.lucide-check')).not.toBeNull();
		await expect.element(page.getByText('Approved by you · sent 14:06')).toBeVisible();
		await expect.element(page.getByRole('link', { name: 'View in Gmail' })).toHaveAttribute('target', '_blank');
		expect(page.getByRole('button').elements()).toHaveLength(0);
	});

	it('says plainly nothing happened when you declined, and lets you undo', async () => {
		const onundo = vi.fn();
		const { container } = await render(ApprovalCard, {
			agent: 'tara',
			title: "Tara's email to Bu Sri",
			status: 'rejected',
			outcome: 'You chose not to send · nothing left your Gmail',
			action: { label: 'Undo', onclick: onundo },
		});
		expect(container.querySelector('svg.lucide-x')).not.toBeNull();
		expect(getComputedStyle(page.getByText("Tara's email to Bu Sri").element()).color).toBe(tokenColor('nm-text-2'));
		await page.getByRole('button', { name: 'Undo' }).click();
		expect(onundo).toHaveBeenCalledOnce();
	});
});

describe('DecisionCard', () => {
	const options = [
		{ title: '3 days before', detail: '25th, 09:00. Time to top up BCA if needed.', suggested: true },
		{ title: 'On the day', detail: '28th, 07:00. Least noise, no buffer.' },
		{ title: 'Both', detail: '25th and 28th. Two pings a month.' },
	];

	it('is one choice of many with the tradeoffs written out, the suggestion picked first', async () => {
		const onconfirm = vi.fn();
		await render(DecisionCard, { question: 'When should Kala remind you about rent?', options, onconfirm });
		const group = page.getByRole('radiogroup', { name: 'When should Kala remind you about rent?' });
		await expect.element(group).toBeVisible();
		await expect.element(page.getByRole('radio', { name: /3 days before/ })).toBeChecked();
		await expect.element(page.getByText('Suggested')).toBeVisible();
		const chosen = page.getByText('3 days before', { exact: true }).element().closest('[data-choice]')!;
		expect(getComputedStyle(chosen).borderColor).toBe(tokenColor('nm-accent'));
		await page.getByRole('radio', { name: /On the day/ }).click();
		await page.getByRole('button', { name: 'Use On the day' }).click();
		expect(onconfirm).toHaveBeenCalledWith(1);
		await expect.element(page.getByText('Or type your own answer')).toBeVisible();
	});

	it('moves between choices with the arrow keys, like any radio group', async () => {
		await render(DecisionCard, { question: 'When?', options, onconfirm: () => {} });
		await page.getByRole('radio', { name: /3 days before/ }).click();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('radio', { name: /On the day/ })).toBeChecked();
	});
});

describe('SelectionCard', () => {
	const items = [
		{ name: 'Netflix Premium', detail: 'monthly · BCA debit', amount: '186.000' },
		{ name: 'Spotify Duo', detail: 'monthly · GoPay', amount: '54.990' },
		{ name: 'Fitness First', detail: 'monthly · since Mar', amount: '650.000' },
	];

	it('is many of many with a running count, and select all', async () => {
		const onconfirm = vi.fn();
		await render(SelectionCard, { agent: 'dana', question: 'Which of these should Dana track as subscriptions?', items, selected: [true, true, false], total: 'Rp 240.990 / month', confirmLabel: 'Track 2', onconfirm, onskip: () => {} });
		await expect.element(page.getByRole('checkbox', { name: /Netflix Premium/ })).toBeChecked();
		await expect.element(page.getByRole('checkbox', { name: /Fitness First/ })).not.toBeChecked();
		await expect.element(page.getByText('2 selected · Rp 240.990 / month')).toBeVisible();
		expect(getComputedStyle(page.getByText('650.000').element()).color).toBe(tokenColor('nm-text-3'));
		await page.getByRole('button', { name: 'Select all' }).click();
		await expect.element(page.getByRole('checkbox', { name: /Fitness First/ })).toBeChecked();
		await expect.element(page.getByRole('button', { name: 'Clear' })).toBeVisible();
		await page.getByRole('button', { name: 'Track 2' }).click();
		expect(onconfirm).toHaveBeenCalledWith([0, 1, 2]);
	});
});

describe('FormCard', () => {
	it('arrives prefilled inline and submits as a form', async () => {
		const onsubmit = vi.fn();
		const oncancel = vi.fn();
		await render(FormCardFixture, { onsubmit, oncancel });
		await expect.element(page.getByText('prefilled from chat')).toBeVisible();
		await expect.element(page.getByRole('textbox', { name: 'What' })).toHaveValue('Pay kos rent to Bu Sri');
		await page.getByRole('button', { name: 'Save reminder' }).click();
		expect(onsubmit).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(oncancel).toHaveBeenCalledOnce();
	});
});

describe('Suggestions', () => {
	it('offers follow-ups as optional next questions', async () => {
		const onpick = vi.fn();
		const { container } = await render(FollowUps, { items: ['Compare with the last three months', 'Set a weekly food budget of Rp 500rb'], onpick });
		await expect.element(page.getByText('Follow up')).toBeVisible();
		expect(container.querySelectorAll('svg.lucide-corner-down-right')).toHaveLength(2);
		await page.getByRole('button', { name: 'Set a weekly food budget of Rp 500rb' }).click();
		expect(onpick).toHaveBeenCalledWith('Set a weekly food budget of Rp 500rb');
	});

	it('answers the last question with one tap', async () => {
		const onpick = vi.fn();
		await render(QuickReplies, { replies: ['Yes, set it', 'Change the time', 'Not now'], onpick });
		const reply = page.getByRole('button', { name: 'Change the time' });
		expect(getComputedStyle(reply.element()).borderRadius).not.toBe('6px');
		await reply.click();
		expect(onpick).toHaveBeenCalledWith('Change the time');
	});
});

const questions = [
	{
		topic: 'Getting there',
		title: 'How do you want to get to Bandung?',
		helper: 'You and Sari, from Jakarta.',
		options: [
			{ title: 'Whoosh high-speed train', detail: '45 min from Halim.', recommended: true },
			{ title: 'Drive', detail: '3–4 hours with weekend traffic.' },
		],
		other: true,
	},
	{
		topic: 'What to cover',
		title: 'What should the plan cover?',
		multiple: true,
		options: [{ title: 'Train tickets' }, { title: 'Where to stay' }, { title: 'Reminders' }],
	},
];

describe('ClarifyCard', () => {
	it('asks one question at a time, numbered, the recommendation picked first', async () => {
		await render(ClarifyCard, { agent: 'nomi', title: 'Before I plan: 2 quick questions', questions, onfinish: () => {}, onskipall: () => {} });
		await expect.element(page.getByText('1 of 2')).toBeVisible();
		await expect.element(page.getByRole('heading', { name: 'How do you want to get to Bandung?' })).toBeVisible();
		await expect.element(page.getByRole('radio', { name: /Whoosh high-speed train/ })).toBeChecked();
		await expect.element(page.getByText('Recommended')).toBeVisible();
		await expect.element(page.getByRole('textbox', { name: 'Something else…' })).toBeVisible();
		await expect.element(page.getByText('1–3 to pick · ↵ next · esc skip all')).toBeVisible();
		const current = page.getByRole('button', { name: 'Getting there' });
		await expect.element(current).toHaveAttribute('aria-current', 'step');
		expect(page.getByRole('button', { name: 'What to cover' }).element().querySelector('svg.lucide-check')).toBeNull();
	});

	it('marks a question answered only once you have been to it', async () => {
		const withDefaults = [questions[0], { ...questions[0], topic: 'Budget' }];
		await render(ClarifyCard, { agent: 'nomi', title: 'Before I plan', questions: withDefaults, onfinish: () => {}, onskipall: () => {} });
		expect(page.getByRole('button', { name: 'Budget' }).element().querySelector('svg.lucide-check')).toBeNull();
		await page.getByRole('button', { name: 'Next' }).click();
		expect(page.getByRole('button', { name: 'Getting there' }).element().querySelector('svg.lucide-check')).not.toBeNull();
	});

	it('picks by number, goes on, and gathers many-of-many answers to review', async () => {
		const onfinish = vi.fn();
		await render(ClarifyCard, { agent: 'nomi', title: 'Before I plan', questions, onfinish, onskipall: () => {} });
		await page.getByRole('radio', { name: /Drive/ }).click();
		await userEvent.keyboard('1');
		await expect.element(page.getByRole('radio', { name: /Whoosh/ })).toBeChecked();
		await userEvent.keyboard('2');
		await expect.element(page.getByRole('radio', { name: /Drive/ })).toBeChecked();
		await page.getByRole('button', { name: 'Next' }).click();
		await expect.element(page.getByText('2 of 2')).toBeVisible();
		await expect.element(page.getByText('0 selected · space to toggle · ↵ review')).toBeVisible();
		await page.getByRole('checkbox', { name: /Train tickets/ }).click();
		await page.getByRole('checkbox', { name: /Reminders/ }).click();
		await expect.element(page.getByText('2 selected · space to toggle · ↵ review')).toBeVisible();
		const done = page.getByRole('button', { name: 'Getting there' }).element();
		expect(done.querySelector('svg.lucide-check')).not.toBeNull();
		await page.getByRole('button', { name: 'Review answers' }).click();
		expect(onfinish).toHaveBeenCalledWith([{ choice: 1 }, { choices: [0, 2] }]);
	});

	it('goes back, skips a question, or skips them all with escape', async () => {
		const onfinish = vi.fn();
		const onskipall = vi.fn();
		await render(ClarifyCard, { agent: 'nomi', title: 'Before I plan', questions, onfinish, onskipall });
		await expect.element(page.getByRole('button', { name: 'Back' })).toBeDisabled();
		await page.getByRole('button', { name: 'Skip' }).click();
		await expect.element(page.getByText('2 of 2')).toBeVisible();
		await page.getByRole('button', { name: 'Back' }).click();
		await expect.element(page.getByText('1 of 2')).toBeVisible();
		await page.getByRole('textbox', { name: 'Something else…' }).fill('Fly');
		await page.getByRole('button', { name: 'What to cover' }).click();
		await page.getByRole('button', { name: 'Review answers' }).click();
		expect(onfinish).toHaveBeenCalledWith([{ other: 'Fly' }, { choices: [] }]);
		await page.getByRole('checkbox', { name: /Train tickets/ }).click();
		await userEvent.keyboard('{Escape}');
		expect(onskipall).toHaveBeenCalledOnce();
	});
});

describe('ClarifyReview', () => {
	it('lists your answers, marks a default it picked and says why, then makes the plan', async () => {
		const onedit = vi.fn();
		const onconfirm = vi.fn();
		await render(ClarifyReview, {
			answers: [
				{ topic: 'Getting there', value: 'Whoosh high-speed train' },
				{ topic: 'Dates', value: 'Sat 1 – Sun 2 Nov', isDefault: true },
			],
			assumption: 'You skipped Dates, so I picked the next free weekend.',
			onedit,
			onback: () => {},
			onconfirm,
		});
		await expect.element(page.getByText('Your answers')).toBeVisible();
		await expect.element(page.getByText('2 of 2')).toBeVisible();
		await expect.element(page.getByText('Default')).toBeVisible();
		await expect.element(page.getByText('You skipped Dates, so I picked the next free weekend.')).toBeVisible();
		await expect.element(page.getByText('Nothing runs yet. You’ll see the plan first.')).toBeVisible();
		await page.getByRole('button', { name: 'Change Dates' }).click();
		expect(onedit).toHaveBeenCalledWith(1);
		await page.getByRole('button', { name: 'Make the plan' }).click();
		expect(onconfirm).toHaveBeenCalledOnce();
	});

	it('folds to one line once answered', async () => {
		const onopen = vi.fn();
		await render(ClarifyAnswered, { count: 4, summary: 'Whoosh · Rp 3,5 jt · 1–2 Nov · 5 topics', onopen });
		const row = page.getByRole('button', { name: /Answered 4 questions/ });
		await expect.element(row).toBeVisible();
		expect(getComputedStyle(page.getByText('Whoosh · Rp 3,5 jt · 1–2 Nov · 5 topics').element()).fontFamily).toContain('Geist Mono');
		await row.click();
		expect(onopen).toHaveBeenCalledOnce();
	});
});

describe('PlanApproval', () => {
	const options = [
		{ title: 'Run it, ask before anything leaves Nomi', detail: 'Research runs freely.', recommended: true },
		{ title: 'Research only', detail: 'Nothing is drafted, shared or tracked.' },
	];

	it('asks how to run the plan, the recommendation first', async () => {
		const onrun = vi.fn();
		await render(PlanApproval, { options, onrun });
		await expect.element(page.getByText('Ready to run this plan?')).toBeVisible();
		await expect.element(page.getByRole('radio', { name: /Run it, ask before anything leaves Nomi/ })).toBeChecked();
		await expect.element(page.getByText('1–3 to pick · ↵ confirm')).toBeVisible();
		const chosen = page.getByText('Research only').element().closest('[data-choice]')!;
		await page.getByRole('radio', { name: /Research only/ }).click();
		expect(getComputedStyle(chosen).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		await page.getByRole('button', { name: 'Run plan' }).click();
		expect(onrun).toHaveBeenCalledWith({ choice: 1 });
	});

	it('keeps planning with what you want changed, and says so on the button', async () => {
		const onrun = vi.fn();
		await render(PlanApproval, { options, onrun });
		await page.getByRole('textbox', { name: 'Tell Nomi what to change…' }).fill('Add a museum on Sunday');
		await expect.element(page.getByRole('radio', { name: /Keep planning/ })).toBeChecked();
		await page.getByRole('button', { name: 'Send changes' }).click();
		expect(onrun).toHaveBeenCalledWith({ keepPlanning: 'Add a museum on Sunday' });
	});
});

describe('ClarifyCard reopened', () => {
	it('starts where it is told, with the answers so far', async () => {
		await render(ClarifyCard, { agent: 'nomi', title: 'Before I plan', questions, step: 1, answers: [{ choice: 1 }], onfinish: () => {}, onskipall: () => {} });
		await expect.element(page.getByText('2 of 2')).toBeVisible();
		expect(page.getByRole('button', { name: 'Getting there' }).element().querySelector('svg.lucide-check')).not.toBeNull();
	});
});
