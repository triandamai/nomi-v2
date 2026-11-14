<script lang="ts">
	import { SearchX, Users } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { UserSummary } from '$lib/admin/api';
	import { compact, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FilterBar from '$lib/ds/admin/FilterBar.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Pager from '$lib/ds/admin/Pager.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Users & access (user: view): everyone on Nomi, twenty a page, searched by email in the address
	// (the backend searches). Each row: the email (to their page), the plan (and an allowance of
	// their own), what they used this month on Nomi's models, their workspaces and any admin role.
	let { data } = $props();

	const language = getLocale();

	const columns: TableColumn<UserSummary>[] = [
		{ key: 'email', label: m.admin_col_email(), sort: (row) => row.email },
		{ key: 'plan', label: m.admin_col_plan(), sort: (row) => row.plan_name, phone: false },
		{ key: 'used', label: m.admin_col_used(), align: 'end', sort: (row) => row.tokens_used },
		{ key: 'workspaces', label: m.admin_col_workspaces(), align: 'end', sort: (row) => row.org_count, phone: false },
		{ key: 'admin', label: m.admin_col_admin(), phone: false },
	];

	/** A page of the list, keeping the search. */
	function pageHref(to: number): string {
		const params = new URLSearchParams(page.url.searchParams);
		params.set('page', String(to));
		return `?${params}`;
	}
</script>

<PageHeader title={m.admin_page_users()} intro={m.admin_users_intro()} />

<FilterBar label={m.admin_users_search()} placeholder={m.admin_users_search_placeholder()} query={data.query} submitLabel={m.admin_search_submit()} />

{#if data.list.ok}
	{@const list = data.list.data}
	{#if list.users.length === 0}
		{#if data.query}
			<EmptyState icon={SearchX} title={m.admin_users_none_title({ query: data.query })} body={m.admin_users_none_body()} />
		{:else}
			<EmptyState icon={Users} title={m.admin_users_empty_title()} body={m.admin_users_empty_body()} />
		{/if}
	{:else}
		{@const from = (data.page - 1) * data.pageSize + 1}
		<AdminTable label={m.admin_users_table()} {columns} rows={list.users} rowKey={(row) => row.id}>
			{#snippet cell(row, column)}
				{#if column.key === 'email'}
					<a href="/admin/users/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{row.email}</a>
				{:else if column.key === 'plan'}
					<span class="flex flex-wrap items-center gap-1.5">
						{row.plan_name}
						{#if row.custom_quota}<span class="font-mono text-label text-text-3">{m.admin_users_own_allowance()}</span>{/if}
					</span>
				{:else if column.key === 'used'}
					<span class="font-mono" title={full(row.tokens_used, language)}>{m.admin_users_used({ used: compact(row.tokens_used, language), total: compact(row.monthly_tokens, language) })}</span>
				{:else if column.key === 'workspaces'}
					<span class="font-mono">{full(row.org_count, language)}</span>
				{:else if column.key === 'admin'}
					{#if row.is_platform_admin}<Tag kind="neutral" label={m.admin_users_platform()} />{:else if row.is_staff}<Tag kind="neutral" label={m.admin_users_areas()} />{/if}
				{/if}
			{/snippet}
		</AdminTable>
		<Pager
			label={m.admin_users_pages()}
			summary={m.admin_pager_summary({ from: full(from, language), to: full(from + list.users.length - 1, language), total: full(list.total, language) })}
			previousLabel={m.admin_previous()}
			nextLabel={m.admin_next()}
			previousHref={data.page > 1 ? pageHref(data.page - 1) : undefined}
			nextHref={from + list.users.length - 1 < list.total ? pageHref(data.page + 1) : undefined}
		/>
	{/if}
{:else}
	<LoadFailed title={m.admin_users_failed()} problem={data.list.problem} />
{/if}
