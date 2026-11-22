<script lang="ts">
	import { Plus, SearchX, Ticket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { AdminCoupon } from '$lib/admin/api';
	import { CURRENCY, money } from '$lib/admin/billing';
	import { dateOnly, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FilterBar from '$lib/ds/admin/FilterBar.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Coupons (plans: view): the codes people can enter at checkout, what each takes off, how often
	// it's been used, when it's valid and which plans it's for. With write: add one; a coupon opens
	// to change it.
	let { data } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'plans', 'write'));

	const discount = (row: AdminCoupon) => (row.discount_kind === 'percent' ? `${row.discount_value}%` : money(row.discount_value, CURRENCY, language));
	function validity(row: AdminCoupon): string {
		if (row.starts_at && row.ends_at) return m.admin_coupon_valid_range({ from: dateOnly(row.starts_at, language, 'UTC'), to: dateOnly(row.ends_at, language, 'UTC') });
		if (row.ends_at) return m.admin_coupon_valid_until({ date: dateOnly(row.ends_at, language, 'UTC') });
		if (row.starts_at) return m.admin_coupon_valid_from({ date: dateOnly(row.starts_at, language, 'UTC') });
		return m.admin_coupon_always();
	}
	const planNames = (row: AdminCoupon) => (row.plan_ids.length ? row.plan_ids.map((id) => data.plans.find((plan) => plan.id === id)?.name ?? id).join(', ') : m.admin_coupon_any_plan());

	const columns: TableColumn<AdminCoupon>[] = [
		{ key: 'code', label: m.admin_coupon_col_code(), sort: (row) => row.code },
		{ key: 'discount', label: m.admin_coupon_col_discount(), align: 'end', sort: (row) => row.discount_value },
		{ key: 'used', label: m.admin_coupon_col_used(), align: 'end', sort: (row) => row.redemptions },
		{ key: 'valid', label: m.admin_coupon_col_valid(), phone: false },
		{ key: 'plans', label: m.admin_coupon_col_plans(), phone: false },
		{ key: 'status', label: m.admin_col_status(), phone: false },
	];
</script>

<PageHeader title={m.admin_page_coupons()} intro={m.admin_coupons_intro()}>
	{#snippet actions()}
		{#if canWrite}<Button label={m.admin_coupon_add()} variant="inverse" iconStart={Plus} href="/admin/billing/coupons/new" />{/if}
	{/snippet}
</PageHeader>

{#if data.total > 0}
	<FilterBar label={m.admin_coupons_search()} placeholder={m.admin_coupons_search_placeholder()} query={data.query} submitLabel={m.admin_search_submit()} />
{/if}

{#if data.coupons.ok}
	{#if data.coupons.data.length === 0}
		{#if data.query}
			<EmptyState icon={SearchX} title={m.admin_coupons_none_title({ query: data.query })} body={m.admin_coupons_none_body()} />
		{:else}
			<EmptyState icon={Ticket} title={m.admin_coupons_empty_title()} body={m.admin_coupons_empty_body()} />
		{/if}
	{:else}
		<AdminTable label={m.admin_coupons_table()} {columns} rows={data.coupons.data} rowKey={(row) => row.id} muted={(row) => !row.is_active}>
			{#snippet cell(row, column)}
				{#if column.key === 'code'}
					<span class="flex min-w-0 flex-col">
						<a href="/admin/billing/coupons/{row.id}" class="rounded-xs font-mono font-medium text-text outline-offset-2 hover:underline">{row.code}</a>
						{#if row.description}<span class="text-caption text-text-3">{row.description}</span>{/if}
					</span>
				{:else if column.key === 'discount'}
					<span class="font-mono">{discount(row)}</span>
				{:else if column.key === 'used'}
					<span class="font-mono">{row.max_redemptions === null ? full(row.redemptions, language) : m.admin_coupon_used_of({ used: full(row.redemptions, language), cap: full(row.max_redemptions, language) })}</span>
				{:else if column.key === 'valid'}
					<span class="text-text-2">{validity(row)}</span>
				{:else if column.key === 'plans'}
					<span class="text-text-2">{planNames(row)}{#if row.min_months > 1} · {m.admin_coupon_from_months({ count: String(row.min_months) })}{/if}{#if row.first_payment_only} · {m.admin_coupon_first_only_short()}{/if}</span>
				{:else if column.key === 'status'}
					<Tag kind="neutral" label={row.is_active ? m.admin_coupon_active() : m.admin_coupon_inactive()} />
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_coupons_failed()} problem={data.coupons.problem} />
{/if}
