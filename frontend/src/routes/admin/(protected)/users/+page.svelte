<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconMore from '$lib/components/icons/IconMore.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
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
				<td>{user.email}</td>
				<td>{user.is_staff ? 'Yes' : 'No'}</td>
				<td>{user.org_count}</td>
				<td>
					<Menu>
						{#snippet trigger({ toggle })}
							<IconButton onclick={toggle} aria-label="Actions for {user.email}">
								<IconMore size={18} />
							</IconButton>
						{/snippet}
						<MenuItem onclick={() => goto(`/admin/users/${user.id}/profile`)}>Update user</MenuItem>
						<MenuItem onclick={() => goto(`/admin/users/${user.id}/role`)}>Update role</MenuItem>
						{#if !user.is_staff}
							<form method="POST" action="?/promote" use:enhance>
								<input type="hidden" name="userId" value={user.id} />
								<MenuItem type="submit">Promote to staff</MenuItem>
							</form>
						{/if}
					</Menu>
				</td>
			</tr>
		{/each}
	</DataTable>
	{#if data.users.length === 0}
		<p class="md-body-medium mt-4" style="color: var(--md-sys-color-on-surface-variant)">No users found.</p>
	{/if}
</div>
