import type { AdminPaymentPage, PaymentStatus } from '$lib/admin/api';
import { PAYMENT_STATUSES } from '$lib/admin/billing';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** Every payment, newest first (plans: view): `?status=` keeps one state, `?page=` the page. */
export const load: PageServerLoad = async (event) => {
	const asked = event.url.searchParams.get('status') ?? '';
	const status = PAYMENT_STATUSES.includes(asked as PaymentStatus) ? asked : '';
	const number = Number.parseInt(event.url.searchParams.get('page') ?? '1', 10);
	const page = Number.isFinite(number) && number > 0 ? number : 1;
	const params = new URLSearchParams({ page: String(page) });
	if (status) params.set('status', status);
	return { status, page, payments: mustLoad(await getJson<AdminPaymentPage>(event, `/api/admin/payments?${params}`)) };
};
