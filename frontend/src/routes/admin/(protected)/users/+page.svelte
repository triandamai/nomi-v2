<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const columns = [
		{ key: 'email', label: 'Email' },
		{ key: 'staff', label: 'Staff' },
		{ key: 'orgs', label: 'Organizations' },
		{ key: 'actions', label: '' },
	];

	function handleSearch(query: string) {
		goto(`?query=${encodeURIComponent(query)}&page=1`, { keepFocus: true });
	}

	function handlePageChange(nextPage: number) {
		const params = new URLSearchParams({ page: String(nextPage) });
		if (data.query) params.set('query', data.query);
		goto(`?${params}`, { keepFocus: true });
	}
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Users</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Manage user access: promote to staff, grant permissions, assign organizations.
</p>

<div class="mt-6">
	<DataTable
		{columns}
		page={data.page}
		pageSize={data.pageSize}
		totalItems={data.total}
		onPageChange={handlePageChange}
		searchQuery={data.query}
		onSearch={handleSearch}
		searchPlaceholder="Search by email..."
	>
		{#each data.users as user (user.id)}
			<tr>
				<td><a href="/admin/users/{user.id}" style="color: var(--md-sys-color-primary)">{user.email}</a></td>
				<td>{user.is_staff ? 'Yes' : 'No'}</td>
				<td>{user.org_count}</td>
				<td>
					{#if !user.is_staff}
						<form method="POST" action="?/promote" use:enhance>
							<input type="hidden" name="userId" value={user.id} />
							<Button type="submit" variant="text">Promote to staff</Button>
						</form>
					{/if}
				</td>
			</tr>
		{/each}
	</DataTable>
	{#if data.users.length === 0}
		<p class="md-body-medium mt-4" style="color: var(--md-sys-color-on-surface-variant)">No users found.</p>
	{/if}
</div>
