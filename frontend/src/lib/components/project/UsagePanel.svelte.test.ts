// @vitest-environment jsdom
// Tokens & cost: the month's spend, tokens and calls, by model and (for admins) by person; since
// when it counts; an empty month; a failed load that can be tried again; and the month picker.
import { describe, expect, it } from 'vitest';
import { fakeBackend, render, settle, text } from '../../../test-stubs/render';
import UsagePanel from './UsagePanel.svelte';

const usage = (over: Record<string, unknown> = {}) => ({
	month: '2026-11',
	current_month: '2026-11',
	timezone: 'Asia/Jakarta',
	counted_since: '2026-11-17T03:00:00Z',
	tokens: 1_350_000,
	input_tokens: 1_250_000,
	output_tokens: 100_000,
	cached_input_tokens: 0,
	own_key_tokens: 50_000,
	calls: 3,
	spend_usd: 5.1,
	by_model: [
		{ admin_model_id: 'm1', label: 'Nomi Standard', provider: 'anthropic', model_id: 'claude-x', source: 'nomi', tokens: 1_300_000, calls: 2, spend_usd: 5.1 },
		{ admin_model_id: null, label: 'My Claude key', provider: 'anthropic', model_id: 'claude-x', source: 'own_key', tokens: 50_000, calls: 1, spend_usd: 0 },
	],
	by_person: [
		{ user_id: 'u1', name: 'Budi Santoso', tokens: 1_100_000, calls: 1, spend_usd: 4.5 },
		{ user_id: 'u2', name: null, tokens: 250_000, calls: 2, spend_usd: 0.6 },
	],
	shows_people: true,
	usd_to_idr: 16_000,
	...over,
});

describe('UsagePanel', () => {
	it("shows the month's spend in rupiah, its models and who spent it", async () => {
		const calls = fakeBackend([['GET', /\/api\/usage/, usage()]]);
		const root = render(UsagePanel as never, { projectId: 'p1' });
		await settle(6);
		expect(calls[0].path).toBe('/projects/p1/api/usage');
		const shown = text(root);
		expect(shown).toContain('Rp');
		expect(shown).toContain('Nomi Standard');
		expect(shown).toContain('Own key');
		expect(shown).toContain('not billed');
		expect(shown).toContain('Budi Santoso');
		expect(shown).toContain('Someone who left');
		expect(shown).toContain('Counted since');
	});

	it('leaves people out for someone who is not an admin', async () => {
		fakeBackend([['GET', /\/api\/usage/, usage({ shows_people: false, by_person: [] })]]);
		const root = render(UsagePanel as never, { projectId: 'p1' });
		await settle(6);
		expect(text(root)).not.toContain('By person');
	});

	it('says when nothing was used, and when counting has not started yet', async () => {
		fakeBackend([['GET', /\/api\/usage/, usage({ calls: 0, tokens: 0, spend_usd: 0, own_key_tokens: 0, by_model: [], by_person: [], counted_since: null })]]);
		const root = render(UsagePanel as never, { projectId: 'p1' });
		await settle(6);
		expect(text(root)).toContain('No tokens used in');
		expect(text(root)).toContain('Counting starts');
	});

	it('goes back a month, and never past this one', async () => {
		const calls = fakeBackend([
			['GET', /usage\?month=2026-10/, usage({ month: '2026-10' })],
			['GET', /\/api\/usage$/, usage()],
		]);
		const root = render(UsagePanel as never, { projectId: 'p1' });
		await settle(6);
		const next = root.querySelector<HTMLButtonElement>('[aria-label="Next month"]')!;
		expect(next.disabled).toBe(true);
		root.querySelector<HTMLButtonElement>('[aria-label="Previous month"]')!.click();
		await settle(6);
		expect(calls.at(-1)!.path).toBe('/projects/p1/api/usage?month=2026-10');
		expect(root.querySelector<HTMLButtonElement>('[aria-label="Next month"]')!.disabled).toBe(false);
	});

	it('offers to try again when the usage could not be loaded', async () => {
		let fail = true;
		fakeBackend([['GET', /\/api\/usage/, () => (fail ? new Response('boom', { status: 500 }) : new Response(JSON.stringify(usage()), { status: 200, headers: { 'content-type': 'application/json' } }))]]);
		const root = render(UsagePanel as never, { projectId: 'p1' });
		await settle(6);
		expect(text(root)).toContain("Couldn’t load this project’s usage.");
		fail = false;
		[...root.querySelectorAll('button')].find((b) => b.textContent?.includes('Try again'))!.click();
		await settle(6);
		expect(text(root)).toContain('Nomi Standard');
	});
});
