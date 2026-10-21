// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount } from 'svelte';
import PlanBlock from './PlanBlock.svelte';
import { refreshPlans } from '$lib/planRefresh.svelte';

type Version = { id: string; title: string; version: number; state?: string; checklist: { text: string; status: string }[] };

function version(v: number, state?: string, checklist = [{ text: 'Pick dates', status: 'in_progress' }]): Version {
	return { id: `p${v}`, title: 'Bali trip', version: v, state, checklist };
}

function respond(...bodies: unknown[]) {
	const fetchMock = vi.fn();
	for (const body of bodies) {
		fetchMock.mockResolvedValueOnce(new Response(JSON.stringify(body), { status: 200, headers: { 'content-type': 'application/json' } }));
	}
	vi.stubGlobal('fetch', fetchMock);
	return fetchMock;
}

let component: ReturnType<typeof mount> | null = null;

async function render(blockVersion: number) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(PlanBlock, {
		target,
		props: { block: { kind: 'plan', plan_id: `p${blockVersion}`, agent_session_id: 'rena-1', title: 'Bali trip', version: blockVersion }, agent: 'Rena' },
	});
	flushSync();
	await vi.waitFor(() => expect(target.textContent).toContain('Pick dates'));
	return target;
}

const status = (target: HTMLElement) => target.querySelector('[role="status"]')?.textContent?.replace(/\s+/g, ' ').trim() ?? null;

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
	vi.unstubAllGlobals();
	vi.useRealTimers();
});

describe('PlanBlock', () => {
	it('shows a plan from a server that doesn’t report states as before, with no status line', async () => {
		respond({ plans: [version(1)] });
		const target = await render(1);
		expect(target.textContent).toContain('Bali trip');
		expect(status(target)).toBeNull();
	});

	it('shows a trip plan nobody has acted on as proposed', async () => {
		respond({ plans: [version(1, 'proposed')], build: null });
		expect(status(await render(1))).toBe('Proposed');
	});

	it('marks an older version as updated, pointing at the latest', async () => {
		respond({ plans: [version(1, 'updated'), version(2, 'proposed')], build: null });
		expect(status(await render(1))).toBe('Updated · Version 2 is the latest');
	});

	it('follows the build live: in progress, then built with the checks it passed', async () => {
		const verified = {
			status: 'verified',
			files_version: 3,
			changed_files: ['src/routes/+page.svelte'],
			deleted_files: [],
			checks: [{ command: 'npm run check', outcome: 'passed', source: 'koda', exit_code: 0, output_tail: null }],
		};
		const fetchMock = respond(
			{ plans: [version(1, 'active')], build: { delegation_id: 'd1', status: 'processing', report: null, error: null, updated_at: '' } },
			{ plans: [version(1, 'completed')], build: { delegation_id: 'd1', status: 'completed', report: verified, error: null, updated_at: '' } },
		);
		const target = await render(1);
		expect(status(target)).toBe('In progress · Koda is building it');

		refreshPlans();
		await vi.waitFor(() => expect(status(target)).toBe('Built · Checks passed on the final files'));
		expect(fetchMock).toHaveBeenCalledTimes(2);
	});
});
