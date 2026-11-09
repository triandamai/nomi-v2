// @vitest-environment jsdom
// The admin overview: the sandbox counts sit beside the agent and token tiles.
import { describe, expect, it } from 'vitest';
import { render, settle, text } from '../../../test-stubs/render';
import Overview from './+page.svelte';

const access = { dashboard: 'manage', user: 'manage', agents: 'manage', models: 'manage', integrations: 'manage' };
const stats = { total_users: 3, tokens_today: 1200, tokens_all_time: 90_000, running_agents: 1, sandbox: { running: 4, starting: 2, error: 1, minutes_this_month: 1234 } };

describe('admin overview', () => {
	it('shows the sandbox running, starting and error counts and the minutes this month', async () => {
		const root = render(Overview as never, { data: { access, stats } as never, form: null as never, params: {} as never });
		await settle(4);
		const tiles = [...root.querySelectorAll('[data-testid="sandbox-stat"]')].map((t) => text(t as HTMLElement));
		expect(tiles).toHaveLength(4);
		expect(tiles[0]).toContain('4');
		expect(tiles[1]).toContain('2');
		expect(tiles[2]).toContain('1');
		expect(tiles[3]).toContain('1.2K');
	});
});
