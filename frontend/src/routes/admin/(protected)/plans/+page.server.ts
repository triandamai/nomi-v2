import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Plan } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export type AdminPlan = Plan & { subscribers: number };

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/plans');
	const plans: AdminPlan[] = response.ok ? ((await response.json()) as AdminPlan[]) : [];
	return { plans };
};

function readPlan(data: FormData) {
	const text = (name: string) => {
		const value = data.get(name);
		return typeof value === 'string' ? value.trim() : '';
	};
	const ends = text('promo_ends_at');
	return {
		slug: text('slug'),
		name: text('name'),
		description: text('description'),
		monthly_tokens: Number(text('monthly_tokens')),
		price_label: text('price_label'),
		features: text('features').split('\n').map((f) => f.trim()).filter(Boolean),
		card_tone: text('card_tone') || 'glow',
		promo_label: text('promo_label') || null,
		promo_price_label: text('promo_price_label') || null,
		// The promo runs to the end of the chosen day.
		promo_ends_at: ends ? new Date(`${ends}T23:59:59`).toISOString() : null,
		is_active: data.get('is_active') === 'true',
		sort_order: Number(text('sort_order')) || 0,
	};
}

async function failure(response: Response, fallback: string) {
	const message = await response.text();
	return fail(response.status, { error: message || fallback });
}

export const actions: Actions = {
	save: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const plan = readPlan(data);
		if (!plan.name || !plan.slug) return fail(400, { error: m.plans_admin_name_required() });
		if (!(plan.monthly_tokens > 0)) return fail(400, { error: m.plans_admin_tokens_required() });
		const editing = typeof id === 'string' && id.length > 0;
		const response = await apiFetch(fetch, cookies, editing ? `/api/admin/plans/${id}` : '/api/admin/plans', {
			method: editing ? 'PUT' : 'POST',
			body: JSON.stringify(plan),
		});
		return response.ok ? { saved: true } : failure(response, m.plans_admin_save_failed());
	},

	setDefault: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, `/api/admin/plans/${id}/default`, { method: 'PUT' });
		return response.ok ? { success: true } : failure(response, m.plans_admin_save_failed());
	},

	delete: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		const response = await apiFetch(fetch, cookies, `/api/admin/plans/${id}`, { method: 'DELETE' });
		return response.ok ? { success: true } : failure(response, m.plans_admin_delete_failed());
	},
};
