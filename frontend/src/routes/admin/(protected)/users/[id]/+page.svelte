<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let open = $state(true);

	$effect(() => {
		// BottomSheet flips `open` to false on its own dismiss affordances (Escape, backdrop
		// click, drag-to-dismiss) — closing this route-driven panel means navigating back.
		if (!open) goto('/admin/users');
	});

	let scopeType = $state<'admin' | 'org'>('admin');
	let grantOrgId = $state(data.orgs[0]?.id ?? '');
	let assignOrgId = $state(data.orgs[0]?.id ?? '');
	let assignRole = $state('member');

	const scopeOptions = [
		{ value: 'admin', label: 'Admin' },
		{ value: 'org', label: 'Organization' },
	];
	const roleOptions = [
		{ value: 'owner', label: 'Owner' },
		{ value: 'admin', label: 'Admin' },
		{ value: 'member', label: 'Member' },
	];
</script>

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{data.user.email}</h2>

	<section class="mt-6">
		<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Permissions</h3>
		{#if data.user.permissions.length === 0}
			<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">No explicit grants.</p>
		{:else}
			<List class="mt-2">
				{#each data.user.permissions as grant (grant.id)}
					<ListItem
						headline="{grant.scope_type === 'admin' ? 'Admin' : (grant.org_name ?? 'Unknown org')}: {grant.resource}"
						supportingText={grant.actions.join(', ')}
					>
						{#snippet trailing()}
							{#if data.canManageUsers}
								<form method="POST" action="?/revokePermission" use:enhance>
									<input type="hidden" name="permissionId" value={grant.id} />
									<IconButton type="submit" aria-label="Revoke {grant.resource}">
										<Icon name="close" size={16} />
									</IconButton>
								</form>
							{/if}
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}

		{#if data.canManageUsers}
			<form method="POST" action="?/grantPermission" use:enhance class="mt-4 flex flex-col gap-3">
				<Select label="Scope" name="scopeType" bind:value={scopeType} options={scopeOptions} />
				{#if scopeType === 'org'}
					<Select
						label="Organization"
						name="orgId"
						bind:value={grantOrgId}
						options={data.orgs.map((o) => ({ value: o.id, label: o.name }))}
					/>
				{/if}
				<TextField id="resource" name="resource" label="Resource" required />
				<div class="flex gap-4">
					<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
						<input type="checkbox" name="actions" value="view" /> View
					</label>
					<label class="md-body-medium flex items-center gap-2" style="color: var(--md-sys-color-on-surface)">
						<input type="checkbox" name="actions" value="manage" /> Manage
					</label>
				</div>
				<Button type="submit" variant="filled" class="w-fit">Grant</Button>
			</form>
		{:else}
			<p class="md-body-small mt-4" style="color: var(--md-sys-color-on-surface-variant)">
				You need manage access to grant permissions.
			</p>
		{/if}
	</section>

	<section class="mt-6">
		<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Organizations</h3>
		{#if data.user.memberships.length === 0}
			<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">Not a member of any organization.</p>
		{:else}
			<List class="mt-2">
				{#each data.user.memberships as membership (membership.org_id)}
					<ListItem headline={membership.org_name} supportingText={membership.role}>
						{#snippet trailing()}
							{#if data.canManageUsers}
								<form method="POST" action="?/removeOrg" use:enhance>
									<input type="hidden" name="orgId" value={membership.org_id} />
									<IconButton type="submit" aria-label="Remove from {membership.org_name}">
										<Icon name="close" size={16} />
									</IconButton>
								</form>
							{/if}
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}

		{#if data.orgs.length > 0 && data.canManageUsers}
			<form method="POST" action="?/assignOrg" use:enhance class="mt-4 flex flex-col gap-3">
				<Select
					label="Organization"
					name="orgId"
					bind:value={assignOrgId}
					options={data.orgs.map((o) => ({ value: o.id, label: o.name }))}
				/>
				<Select label="Role" name="role" bind:value={assignRole} options={roleOptions} />
				<Button type="submit" variant="filled" class="w-fit">Assign</Button>
			</form>
		{:else if data.orgs.length > 0}
			<p class="md-body-small mt-4" style="color: var(--md-sys-color-on-surface-variant)">
				You need manage access to assign organizations.
			</p>
		{/if}
	</section>
</BottomSheet>
