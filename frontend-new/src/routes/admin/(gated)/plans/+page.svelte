<script lang="ts">
	import { CreditCard, Plus, Star } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { AdminPlan } from '$lib/admin/api';
	import { compact, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Plans (plans: view): each plan's monthly tokens on Nomi's models, its price, who is on it, and
	// whether it's the default, offered or hidden. With write: add one, and make one the default
	// (the star). A plan opens to change it.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'plans', 'write'));

	const columns: TableColumn<AdminPlan>[] = [
		{ key: 'name', label: m.admin_col_name(), sort: (row) => row.name },
		{ key: 'slug', label: m.admin_col_slug(), phone: false },
		{ key: 'tokens', label: m.admin_col_allowance(), align: 'end', sort: (row) => row.monthly_tokens },
		{ key: 'price', label: m.admin_col_price(), phone: false },
		{ key: 'people', label: m.admin_col_subscribers(), align: 'end', sort: (row) => row.subscribers },
		{ key: 'status', label: m.admin_col_status(), phone: false },
	];
</script>

<PageHeader title={m.admin_page_plans()} intro={m.admin_plans_intro()}>
	{#snippet actions()}
		{#if canWrite}<Button label={m.admin_plans_add()} variant="inverse" iconStart={Plus} href="/admin/plans/new" />{/if}
	{/snippet}
</PageHeader>

<Outcome {form} action="makeDefault" done={m.admin_plan_default_done()} failed={m.admin_plan_default_failed()} />

{#if data.plans.ok}
	{#if data.plans.data.length === 0}
		<EmptyState icon={CreditCard} title={m.admin_plans_empty_title()} body={m.admin_plans_empty_body()} />
	{:else}
		<AdminTable label={m.admin_plans_table()} {columns} rows={data.plans.data} rowKey={(row) => row.id} muted={(row) => !row.is_active}>
			{#snippet cell(row, column)}
				{#if column.key === 'name'}
					<span class="flex items-center gap-1">
						<a href="/admin/plans/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{row.name}</a>
						{#if canWrite && !row.is_default}
							<form method="POST" action="?/makeDefault" use:enhance>
								<input type="hidden" name="id" value={row.id} />
								<IconButton icon={Star} label={m.admin_plan_make_default({ name: row.name })} variant="ghost" type="submit" />
							</form>
						{/if}
					</span>
				{:else if column.key === 'slug'}
					<span class="font-mono text-text-2">{row.slug}</span>
				{:else if column.key === 'tokens'}
					<span class="font-mono" title={full(row.monthly_tokens, language)}>{compact(row.monthly_tokens, language)}</span>
				{:else if column.key === 'price'}
					{row.promo_price_label ?? row.price_label}
				{:else if column.key === 'people'}
					<span class="font-mono">{full(row.subscribers, language)}</span>
				{:else if column.key === 'status'}
					<span class="flex flex-wrap gap-1.5">
						{#if row.is_default}<Tag kind="neutral" label={m.admin_plan_default()} />{/if}
						<Tag kind="neutral" label={row.is_active ? m.admin_plan_offered() : m.admin_plan_hidden()} />
					</span>
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_plans_failed()} problem={data.plans.problem} />
{/if}
