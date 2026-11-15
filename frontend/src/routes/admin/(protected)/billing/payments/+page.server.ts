import { apiFetch } from '$lib/server/api';
import { PAYMENT_STATUSES } from '$lib/billing';
import type { AdminPaymentPage, PaymentStatus } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const asked = url.searchParams.get('status') ?? '';
	const status = PAYMENT_STATUSES.includes(asked as PaymentStatus) ? asked : '';
	const page = Math.max(1, Math.trunc(Number(url.searchParams.get('page'))) || 1);
	const params = new URLSearchParams({ page: String(page) });
	if (status) params.set('status', status);
	const response = await apiFetch(fetch, cookies, `/api/admin/payments?${params}`);
	const result: AdminPaymentPage = response.ok ? ((await response.json()) as AdminPaymentPage) : { items: [], total: 0, page, page_size: 1 };
	return { ...result, status };
};
