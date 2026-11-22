<script lang="ts">
	import { Receipt } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminPayment } from '$lib/admin/api';
	import { PAYMENT_STATUSES, money, paymentStatusLabel } from '$lib/admin/billing';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Pager from '$lib/ds/admin/Pager.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Payments (plans: view): every payment, newest first: when, who, which plan for how long, how
	// much, its state, when it was paid and how. One state at a time from the filter; the address
	// keeps the filter and the page.
	let { data } = $props();

	const language = getLocale();

	const href = (status: string, page = 1) => {
		const params = new URLSearchParams();
		if (status) params.set('status', status);
		if (page > 1) params.set('page', String(page));
		const query = params.toString();
		return query ? `?${query}` : '/admin/billing/payments';
	};
	const filters = [{ value: '', label: m.admin_payments_all() }, ...PAYMENT_STATUSES.map((status) => ({ value: status, label: paymentStatusLabel(status) }))];

	const columns: TableColumn<AdminPayment>[] = [
		{ key: 'date', label: m.admin_payments_col_date() },
		{ key: 'user', label: m.admin_payments_col_user() },
		{ key: 'plan', label: m.admin_payments_col_plan(), phone: false },
		{ key: 'amount', label: m.admin_payments_col_amount(), align: 'end' },
		{ key: 'status', label: m.admin_col_status() },
		{ key: 'paid', label: m.admin_payments_col_paid(), phone: false },
		{ key: 'method', label: m.admin_payments_col_method(), phone: false },
	];
</script>

<PageHeader title={m.admin_page_payments()} intro={m.admin_payments_intro()} />

<Segmented label={m.admin_payments_filter()} options={filters} value={data.status} onchange={(status) => goto(href(status))} />

{#if data.payments.ok}
	{@const list = data.payments.data}
	{#if list.items.length === 0}
		<EmptyState icon={Receipt} title={data.status ? m.admin_payments_none_title() : m.admin_payments_empty_title()} body={data.status ? m.admin_payments_none_body() : m.admin_payments_empty_body()} />
	{:else}
		{@const from = (list.page - 1) * list.page_size + 1}
		<AdminTable label={m.admin_payments_table()} {columns} rows={list.items} rowKey={(row) => row.id}>
			{#snippet cell(row, column)}
				{#if column.key === 'date'}
					<span class="font-mono text-label text-text-2">{dateTime(row.created_at, language)}</span>
				{:else if column.key === 'user'}
					<span class="text-text">{row.user_email ?? '—'}</span>
				{:else if column.key === 'plan'}
					<span class="text-text-2">{m.admin_payments_plan({ plan: row.plan_name, months: String(row.term_months) })}</span>
				{:else if column.key === 'amount'}
					<span class="font-mono">{money(row.amount, row.currency, language)}</span>
				{:else if column.key === 'status'}
					<Tag kind={row.status === 'paid' ? 'go' : row.status === 'failed' ? 'no-go' : 'neutral'} label={paymentStatusLabel(row.status)} />
				{:else if column.key === 'paid'}
					<span class="font-mono text-label text-text-2">{dateTime(row.paid_at, language)}</span>
				{:else if column.key === 'method'}
					<span class="font-mono text-label text-text-2">{row.payment_method ?? '—'}</span>
				{/if}
			{/snippet}
		</AdminTable>
		<Pager
			label={m.admin_payments_pages()}
			summary={m.admin_pager_summary({ from: full(from, language), to: full(from + list.items.length - 1, language), total: full(list.total, language) })}
			previousLabel={m.admin_previous()}
			nextLabel={m.admin_next()}
			previousHref={list.page > 1 ? href(data.status, list.page - 1) : undefined}
			nextHref={from + list.items.length - 1 < list.total ? href(data.status, list.page + 1) : undefined}
		/>
	{/if}
{:else}
	<LoadFailed title={m.admin_payments_failed()} problem={data.payments.problem} />
{/if}
