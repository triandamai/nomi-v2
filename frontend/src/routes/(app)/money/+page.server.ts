import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MoneySummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

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
	addExpense: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const body = {
			amount: Number(data.get('amount')),
			category: String(data.get('category') ?? '').trim(),
			description: String(data.get('description') ?? '').trim(),
			occurred_at: String(data.get('occurred_at') ?? '') || undefined,
		};
		if (!body.description || !body.category || !(body.amount > 0)) {
			return fail(400, { error: 'Add an amount, what it was for, and a category.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/money/transactions', { method: 'POST', body: JSON.stringify(body) });
		if (!response.ok) return failWith(response, 'Couldn’t add that expense.');
		return { saved: 'expense' };
	},
	setBudget: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const body = { category: String(data.get('category') ?? '').trim(), monthly_limit: Number(data.get('monthly_limit')) };
		if (!body.category || !(body.monthly_limit > 0)) return fail(400, { error: 'Pick a category and a monthly limit above zero.' });
		const response = await apiFetch(fetch, cookies, '/api/money/budgets', { method: 'PUT', body: JSON.stringify(body) });
		if (!response.ok) return failWith(response, 'Couldn’t save that budget.');
		return { saved: 'budget' };
	},
	deleteBudget: async ({ request, cookies, fetch }) => {
		const category = String((await request.formData()).get('category') ?? '');
		const response = await apiFetch(fetch, cookies, `/api/money/budgets/${encodeURIComponent(category)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) return failWith(response, 'Couldn’t remove that budget.');
		return { saved: 'budget-removed' };
	},
};
