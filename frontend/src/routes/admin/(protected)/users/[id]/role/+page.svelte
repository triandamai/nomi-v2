<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Radio from '$lib/components/m3/Radio.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { ADMIN_PERMISSION_RESOURCES, CUSTOM_PERMISSION_RESOURCE } from '$lib/permissions';
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

	let selectedResource = $state(ADMIN_PERMISSION_RESOURCES[0]?.resource ?? CUSTOM_PERMISSION_RESOURCE);
	let customResource = $state('');
	let actionView = $state(false);
	let actionManage = $state(false);

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
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Update role</h2>
	<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{data.user.email}</p>

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
										<IconClose size={16} />
									</IconButton>
								</form>
							{/if}
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}

		{#if data.canManageUsers}
			<form method="POST" action="?/grantPermission" use:enhance class="mt-4 flex flex-col gap-4">
				<Select label="Scope" name="scopeType" bind:value={scopeType} options={scopeOptions} />
				{#if scopeType === 'org'}
					<Select
						label="Organization"
						name="orgId"
						bind:value={grantOrgId}
						options={data.orgs.map((o) => ({ value: o.id, label: o.name }))}
					/>
				{/if}

				<div>
					<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">Resource</span>
					<div class="mt-2 flex flex-col gap-2">
						{#each ADMIN_PERMISSION_RESOURCES as resourceDef (resourceDef.resource)}
							<div class="m3-resource-row">
								<Radio name="resource" value={resourceDef.resource} bind:group={selectedResource} label={resourceDef.label} />
								<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">
									{resourceDef.description}
								</span>
							</div>
						{/each}
						<div class="m3-resource-row">
							<Radio name="resource" value={CUSTOM_PERMISSION_RESOURCE} bind:group={selectedResource} label="Other" />
						</div>
						{#if selectedResource === CUSTOM_PERMISSION_RESOURCE}
							<TextField
								id="customResource"
								name="customResource"
								label="Custom resource"
								bind:value={customResource}
								supportingText="Lowercase letters, digits, and underscores only."
							/>
						{/if}
					</div>
				</div>

				<div>
					<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">Actions</span>
					<div class="mt-2 flex gap-4">
						<Checkbox name="actions" value="view" bind:checked={actionView} label="View" />
						<Checkbox name="actions" value="manage" bind:checked={actionManage} label="Manage" />
					</div>
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
										<IconClose size={16} />
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

<style>
	.m3-resource-row {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 0;
	}
</style>
