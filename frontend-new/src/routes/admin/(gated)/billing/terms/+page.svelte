<script lang="ts">
	import { CalendarRange, Plus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { BillingTerm } from '$lib/admin/api';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Billing terms (plans: view): the lengths of time a plan can be paid for, and what each takes
	// off ("6 months, 10% off"). With write: add one; a term opens to change it.
	let { data } = $props();

	const canWrite = $derived(can(data.access, 'plans', 'write'));

	const columns: TableColumn<BillingTerm>[] = [
		{ key: 'months', label: m.admin_term_col_length(), sort: (row) => row.months },
		{ key: 'label', label: m.admin_term_col_label(), phone: false },
		{ key: 'discount', label: m.admin_term_col_discount(), align: 'end', sort: (row) => row.discount_percent },
		{ key: 'order', label: m.admin_term_col_order(), align: 'end', phone: false, sort: (row) => row.sort_order },
		{ key: 'status', label: m.admin_col_status(), phone: false },
	];
</script>

<PageHeader title={m.admin_page_terms()} intro={m.admin_terms_intro()}>
	{#snippet actions()}
		{#if canWrite}<Button label={m.admin_term_add()} variant="inverse" iconStart={Plus} href="/admin/billing/terms/new" />{/if}
	{/snippet}
</PageHeader>

{#if data.terms.ok}
	{#if data.terms.data.length === 0}
		<EmptyState icon={CalendarRange} title={m.admin_terms_empty_title()} body={m.admin_terms_empty_body()} />
	{:else}
		<AdminTable label={m.admin_terms_table()} {columns} rows={data.terms.data} rowKey={(row) => row.id} muted={(row) => !row.is_active}>
			{#snippet cell(row, column)}
				{#if column.key === 'months'}
					<a href="/admin/billing/terms/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{m.admin_term_months({ count: String(row.months) })}</a>
				{:else if column.key === 'label'}
					<span class="text-text-2">{row.label || '—'}</span>
				{:else if column.key === 'discount'}
					<span class="font-mono">{row.discount_percent > 0 ? `${row.discount_percent}%` : '—'}</span>
				{:else if column.key === 'order'}
					<span class="font-mono text-text-2">{row.sort_order}</span>
				{:else if column.key === 'status'}
					<Tag kind="neutral" label={row.is_active ? m.admin_plan_offered() : m.admin_plan_hidden()} />
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_terms_failed()} problem={data.terms.problem} />
{/if}
