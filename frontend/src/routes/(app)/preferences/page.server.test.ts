import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json } from '../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const bodies = () => apiFetch.mock.calls.map(([, , , init]) => (init?.body ? JSON.parse(init.body) : undefined));
const status = (result: unknown) => (result as { status?: number }).status;

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async () => json({}));
});

describe('preferences', () => {
	it('loads them, with defaults when they can’t be read', async () => {
		apiFetch.mockImplementation(async () => json({ theme: 'dark' }));
		expect(await load(event())).toEqual({ preferences: { theme: 'dark' } });
		apiFetch.mockImplementation(async () => json({}, 500));
		expect(((await load(event())) as { preferences: { theme: string } }).preferences.theme).toBe('system');
	});

	it('saves each setting that is valid and refuses the rest', async () => {
		const good: [keyof typeof actions, Record<string, string>, unknown][] = [
			['updateTheme', { theme: 'dark' }, { theme: 'dark' }],
			['updateTimezone', { timezone: 'Asia/Jakarta' }, { timezone: 'Asia/Jakarta' }],
			['updateBackdrop', { animated_backdrop: 'true' }, { animated_backdrop: true }],
			['updateDrawer', { pins: '[]' }, { drawer_pins: [] }],
		];
		for (const [name, form, body] of good) {
			expect(await actions[name](event({ form }))).toEqual({ success: true });
			expect(bodies().at(-1)).toEqual(body);
		}
		const bad: [keyof typeof actions, Record<string, string>][] = [
			['updateTheme', { theme: 'neon' }],
			['updateAccentColor', { accent_color: 'not-a-theme' }],
			['updateTimezone', { timezone: '' }],
			['updateHomeLayout', { home_layout: 'nope' }],
			['updateDrawer', { pins: 'not json' }],
			['updateDrawer', { pins: '[{"bad": true}]' }],
		];
		for (const [name, form] of bad) expect(status(await actions[name](event({ form }))), name).toBe(400);
	});

	it('passes a failed save on', async () => {
		apiFetch.mockImplementation(async () => json({}, 503));
		for (const [name, form] of [
			['updateTheme', { theme: 'light' }],
			['updateTimezone', { timezone: 'UTC' }],
			['updateBackdrop', {}],
			['updateDrawer', { pins: '[]' }],
		] as [keyof typeof actions, Record<string, string>][]) {
			expect(status(await actions[name](event({ form }))), name).toBe(503);
		}
	});
});
