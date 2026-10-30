import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import Reasoning from './Reasoning.svelte';
import Thought from './Thought.svelte';
import ThoughtSteps from './ThoughtSteps.svelte';
import PlanCard from './PlanCard.svelte';
import DraftPlanCard from './DraftPlanCard.svelte';
import { tokenColor } from '../test-utils';

// Chat responses → Thinking & planning: how the crew shows its work. Collapsed by default once the
// answer lands; live while it runs. Never theatrical.

describe('Reasoning', () => {
	it('streams while it runs, with the time so far', async () => {
		const { container } = await render(Reasoning, { live: true, label: 'Thinking', elapsed: '00:04', text: 'Looking at October in Dana’s records…' });
		expect(container.querySelector('svg.lucide-loader-circle')).not.toBeNull();
		await expect.element(page.getByText('Looking at October in Dana’s records…')).toBeVisible();
		expect(getComputedStyle(page.getByText('00:04').element()).fontFamily).toContain('Geist Mono');
	});

	it('folds away once the answer lands, and opens on request', async () => {
		await render(Reasoning, { live: false, label: 'Thought for 8s', text: 'First thought.\n\nSecond thought.' });
		const toggle = page.getByRole('button', { name: 'Thought for 8s' });
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'false');
		await expect.element(page.getByText('First thought.')).not.toBeInTheDocument();
		await toggle.click();
		await expect.element(toggle).toHaveAttribute('aria-expanded', 'true');
		await expect.element(page.getByText('First thought.')).toBeVisible();
		await expect.element(page.getByText('Second thought.')).toBeVisible();
		expect(getComputedStyle(page.getByText('First thought.').element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('Thought', () => {
	it('notes one decision quietly', async () => {
		const { container } = await render(Thought, { text: "You asked about this month only, so I'll leave last year's data out." });
		expect(container.querySelector('svg.lucide-lightbulb')).not.toBeNull();
		expect(getComputedStyle(page.getByText(/this month only/).element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('ThoughtSteps', () => {
	it('lists the real steps with the tool, size and time of each', async () => {
		const { container } = await render(ThoughtSteps, {
			label: 'Worked through 4 steps',
			meta: '· 12s',
			steps: [
				{ label: 'Read October transactions', meta: 'dana.records · 142 rows · 1.1s', state: 'done' },
				{ label: 'Drafting the answer', meta: 'nomi · now', state: 'running' },
			],
		});
		expect(page.getByRole('listitem').elements()).toHaveLength(2);
		expect(getComputedStyle(page.getByText('dana.records · 142 rows · 1.1s').element()).fontFamily).toContain('Geist Mono');
		expect(container.querySelectorAll('svg.lucide-check')).toHaveLength(1);
		expect(container.querySelectorAll('svg.lucide-loader-circle')).toHaveLength(1);
		await page.getByRole('button', { name: /Worked through 4 steps/ }).click();
		expect(page.getByRole('listitem').elements()).toHaveLength(0);
	});
});

const runningSteps = [
	{ label: 'Pull October transactions', meta: '142 rows', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
	{ label: 'Find the categories that grew', meta: 'food +48% vs September', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
	{ label: 'Draft a weekly food budget', meta: 'running · 00:06', state: 'running', owner: { agent: 'nomi', name: 'Nomi' } },
	{ label: 'Set a Sunday check-in reminder', meta: 'waits for step 3', state: 'waiting', owner: { agent: 'kala', name: 'Kala' } },
] as const;

describe('PlanCard', () => {
	it('shows each numbered step with its real state and owner, and how far along it is', async () => {
		const onstop = vi.fn();
		const { container } = await render(PlanCard, { subject: 'Weekly money check', steps: [...runningSteps], onstop });
		await expect.element(page.getByText('2 of 4 done')).toBeInTheDocument();
		for (const index of ['01', '02', '03', '04']) await expect.element(page.getByText(index, { exact: true })).toBeInTheDocument();
		expect(container.querySelectorAll('svg.lucide-check')).toHaveLength(2);
		expect(container.querySelector('svg.lucide-circle-dashed')).not.toBeNull();
		expect(getComputedStyle(page.getByText('Set a Sunday check-in reminder').element()).color).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByText('Kala', { exact: true })).toBeInTheDocument();
		await page.getByRole('button', { name: 'Stop plan' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('DraftPlanCard', () => {
	const steps = [
		{ label: 'Read the rent transfers in your BCA statements', owner: { agent: 'maya', name: 'Maya' } },
		{ label: "Record each month's rent as an expense", owner: { agent: 'dana', name: 'Dana' } },
		{ label: 'Remind you on the 25th, three days before rent is due', owner: { agent: 'kala', name: 'Kala' } },
	] as const;

	it('lets the person edit before anything runs', async () => {
		const handlers = { onmove: vi.fn(), onremove: vi.fn(), onassign: vi.fn(), onadd: vi.fn(), onrun: vi.fn(), onchanges: vi.fn(), ondiscard: vi.fn() };
		// `intro` is also a Svelte mount option, so the props go under `props`.
		await render(DraftPlanCard, {
			props: {
				intro: "How I'd set up the kos rent tracker. Reorder, reassign or remove steps; nothing runs until you say so.",
				steps: [...steps],
				estimate: '~2 min · Maya, Dana, Kala',
				...handlers,
			},
		});
		await expect.element(page.getByText('Draft', { exact: true })).toBeInTheDocument();
		await page.getByRole('button', { name: 'Remove step 2' }).click();
		expect(handlers.onremove).toHaveBeenCalledWith(1);
		await page.getByRole('button', { name: /Maya/ }).click();
		expect(handlers.onassign).toHaveBeenCalledWith(0);
		await page.getByRole('button', { name: 'Add a step' }).click();
		expect(handlers.onadd).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Run plan' }).click();
		await page.getByRole('button', { name: 'Ask for changes' }).click();
		await page.getByRole('button', { name: 'Discard' }).click();
		expect([handlers.onrun, handlers.onchanges, handlers.ondiscard].map((fn) => fn.mock.calls.length)).toEqual([1, 1, 1]);
		expect(getComputedStyle(page.getByText('~2 min · Maya, Dana, Kala').element()).fontFamily).toContain('Geist Mono');
	});

	it('reorders a step with the keyboard from its handle', async () => {
		const onmove = vi.fn();
		await render(DraftPlanCard, { props: { intro: 'Draft', steps: [...steps], estimate: '~2 min', onmove } });
		await page.getByRole('button', { name: 'Move step 2', exact: true }).click();
		await userEvent.keyboard('{ArrowUp}');
		expect(onmove).toHaveBeenCalledWith(1, 0);
		await userEvent.keyboard('{ArrowDown}');
		expect(onmove).toHaveBeenLastCalledWith(1, 2);
	});
});
