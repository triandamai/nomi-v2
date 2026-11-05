import { describe, expect, it, vi } from 'vitest';
import { load, actions } from './+page.server';

vi.mock('$lib/server/api', () => ({
	apiFetch: vi.fn(async (_fetch, _cookies, path, init) => {
		if (path === '/api/profile' && (!init || init.method === 'GET')) {
			return {
				ok: true,
				status: 200,
				json: async () => ({
					display_name: 'Mario Prasetyo',
					username: 'mario',
					email: 'mario@example.com',
					avatar_url: null,
					location: 'Jakarta',
					joined_at: '2026-07-14T00:00:00Z',
					plan_name: 'Free plan',
					show_stats_publicly: true,
					show_crew_publicly: false,
					showcase_project_ids: [],
					showcase_projects: [],
				}),
			};
		}
		if (path === '/api/projects') {
			return {
				ok: true,
				status: 200,
				json: async () => ({
					projects: [
						{ id: '11111111-1111-1111-1111-111111111111', name: 'salah-daily-tracker', status: 'ready' },
					],
				}),
			};
		}
		if (path === '/api/profile' && init?.method === 'PUT') {
			const body = JSON.parse(init.body as string);
			return {
				ok: true,
				status: 200,
				json: async () => ({
					...body,
					email: 'mario@example.com',
				}),
			};
		}
		return { ok: false, status: 404, text: async () => 'Not found' };
	}),
}));

describe('profile page.server', () => {
	it('loads profile and projects', async () => {
		const mockFetch = vi.fn();
		const mockCookies = {} as any;
		const result = (await load({
			fetch: mockFetch,
			cookies: mockCookies,
		} as any)) as { profile: any; projects: any[] };

		expect(result.profile).toBeDefined();
		expect(result.profile?.username).toBe('mario');
		expect(result.projects).toHaveLength(1);
		expect(result.projects[0].name).toBe('salah-daily-tracker');
	});

	it('updates profile settings through updateProfile action', async () => {
		const mockFetch = vi.fn();
		const mockCookies = {} as any;
		const formData = new FormData();
		formData.append('display_name', 'Mario Prasetyo');
		formData.append('username', '@mario');
		formData.append('location', 'Jakarta');
		formData.append('show_stats_publicly', 'true');
		formData.append('show_crew_publicly', 'true');
		formData.append('showcase_project_ids', JSON.stringify(['11111111-1111-1111-1111-111111111111']));

		const result = (await actions.updateProfile({
			request: {
				formData: async () => formData,
			} as any,
			cookies: mockCookies,
			fetch: mockFetch,
		} as any)) as { success: boolean; profile: any };

		expect(result.success).toBe(true);
		expect(result.profile.username).toBe('mario');
		expect(result.profile.location).toBe('Jakarta');
		expect(result.profile.show_stats_publicly).toBe(true);
		expect(result.profile.show_crew_publicly).toBe(true);
	});
});
