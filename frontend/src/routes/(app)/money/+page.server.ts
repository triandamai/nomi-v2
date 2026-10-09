import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MoneySummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const month = url.searchParams.get('month');
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const response = await apiFetch(fetch, cookies, `/api/money${query}`);
	const money: MoneySummary | null = response.ok ? ((await response.json()) as MoneySummary) : null;
	return { money };
};

async function failWith(response: Response, fallback: string) {
	const text = await response.text();
	return fail(response.status, { error: text && text.length < 140 ? text : fallback });
}

export const actions: Actions = {
	addTransaction: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const kind = data.get('kind') === 'income' ? 'income' : 'expense';
		let items: { name: string; quantity: number; unit_amount: number }[] = [];
		try {
			const parsed = JSON.parse(String(data.get('items_json') ?? '[]'));
			if (Array.isArray(parsed)) items = parsed;
		} catch {
			items = [];
		}
		const amount = Number(data.get('amount'));
		const body = {
			kind,
			// With items, the total is theirs; an amount typed alongside is ignored.
			amount: items.length ? undefined : amount,
			items: items.length ? items : undefined,
			category: String(data.get('category') ?? '').trim(),
			description: String(data.get('description') ?? '').trim(),
			occurred_at: String(data.get('occurred_at') ?? '') || undefined,
		};
		if (!body.description || !body.category || (!items.length && !(amount > 0))) {
			return fail(400, { error: m.money_expense_missing() });
		}
		const response = await apiFetch(fetch, cookies, '/api/money/transactions', { method: 'POST', body: JSON.stringify(body) });
		if (!response.ok) return failWith(response, m.money_expense_failed());
		return { saved: kind };
	},
	setPeriod: async ({ request, cookies, fetch }) => {
		const startDay = Number((await request.formData()).get('start_day'));
		if (!Number.isInteger(startDay) || startDay < 1 || startDay > 28) return fail(400, { error: m.money_period_invalid() });
		const response = await apiFetch(fetch, cookies, '/api/money/period', { method: 'PUT', body: JSON.stringify({ start_day: startDay }) });
		if (!response.ok) return failWith(response, m.money_period_failed());
		return { saved: 'period' };
	},
	setBudget: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const body = { category: String(data.get('category') ?? '').trim(), monthly_limit: Number(data.get('monthly_limit')) };
		if (!body.category || !(body.monthly_limit > 0)) return fail(400, { error: m.money_budget_missing() });
		const response = await apiFetch(fetch, cookies, '/api/money/budgets', { method: 'PUT', body: JSON.stringify(body) });
		if (!response.ok) return failWith(response, m.money_budget_failed());
		return { saved: 'budget' };
	},
	deleteBudget: async ({ request, cookies, fetch }) => {
		const category = String((await request.formData()).get('category') ?? '');
		const response = await apiFetch(fetch, cookies, `/api/money/budgets/${encodeURIComponent(category)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) return failWith(response, m.money_budget_remove_failed());
		return { saved: 'budget-removed' };
	},
};
