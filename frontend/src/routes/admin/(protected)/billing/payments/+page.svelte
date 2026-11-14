<script lang="ts">
	// Admin → Billing → Payments: every payment, newest first, with a filter on its state.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { goto } from '$app/navigation';
	import Chip from '$lib/components/m3/Chip.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import PaymentStatus from '$lib/components/billing/PaymentStatus.svelte';
	import { PAYMENT_STATUSES, formatMoney, paymentStatusLabel } from '$lib/billing';
	import { pageCount } from '$lib/pagination';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const pages = $derived(pageCount(data.total, data.page_size));
	const link = (status: string, page = 1) => {
		const params = new URLSearchParams();
		if (status) params.set('status', status);
		if (page > 1) params.set('page', String(page));
		const query = params.toString();
		return query ? `?${query}` : '?';
	};
	const when = (iso: string | null) =>
		iso ? new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' }).format(new Date(iso)) : '—';
</script>

<PageHeader title={m.billing_admin_payments()} lede={m.billing_admin_payments_page_lede()} agent="money" />

<div class="filters" role="group" aria-label={m.billing_admin_payments_filter()}>
	<Chip variant="filter" selected={data.status === ''} onclick={() => goto(link(''))}>{m.billing_admin_payments_all()}</Chip>
	{#each PAYMENT_STATUSES as status (status)}
		<Chip variant="filter" selected={data.status === status} onclick={() => goto(link(status))}>{paymentStatusLabel(status)}</Chip>
	{/each}
</div>

{#if data.items.length === 0}
	<p class="muted">{m.billing_admin_payments_none()}</p>
{:else}
	<div class="table-scroll">
		<table class="table">
			<thead>
				<tr>
					<th scope="col">{m.billing_admin_payments_date()}</th>
					<th scope="col">{m.billing_admin_payments_user()}</th>
					<th scope="col">{m.billing_payment_plan()}</th>
					<th scope="col" class="num">{m.billing_payment_months()}</th>
					<th scope="col" class="num">{m.billing_payment_amount()}</th>
					<th scope="col">{m.billing_payment_status()}</th>
					<th scope="col">{m.billing_admin_payments_paid()}</th>
					<th scope="col">{m.billing_admin_payments_method()}</th>
				</tr>
			</thead>
			<tbody>
				{#each data.items as payment (payment.id)}
					<tr>
						<td>{when(payment.created_at)}</td>
						<td class="wrap">{payment.user_email ?? '—'}</td>
						<td>{payment.plan_name}</td>
						<td class="num">{payment.term_months}</td>
						<td class="num">{formatMoney(payment.amount, payment.currency, getLocale())}</td>
						<td><PaymentStatus status={payment.status} /></td>
						<td>{when(payment.paid_at)}</td>
						<td>{payment.payment_method ?? '—'}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
	<p class="muted">{m.billing_admin_payments_count({ count: data.total })}</p>
	{#if pages > 1}
		<Pagination page={data.page} {pages} href={(target) => link(data.status, target)} />
	{/if}
{/if}

<style>
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.filters {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.table-scroll {
		overflow-x: auto;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}
	.table th {
		padding: 10px 12px;
		text-align: left;
		font-weight: 600;
		white-space: nowrap;
		color: var(--md-sys-color-on-surface-variant);
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.table td {
		padding: 10px 12px;
		white-space: nowrap;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.table tr:last-child td {
		border-bottom: none;
	}
	.wrap {
		white-space: normal;
		overflow-wrap: anywhere;
	}
	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
</style>
