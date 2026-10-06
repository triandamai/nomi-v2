<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import { deserialize, enhance } from '$app/forms';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconMore from '$lib/components/icons/IconMore.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import Radio from '$lib/components/m3/Radio.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { ADMIN_PERMISSION_RESOURCES, CUSTOM_PERMISSION_RESOURCE } from '$lib/permissions';
	import type { AdminUserDetail } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const columns = [
		{ key: 'email', label: m.users_person() },
		{ key: 'staff', label: m.users_role() },
		{ key: 'orgs', label: m.users_spaces() },
		{ key: 'actions', label: '' },
	];

	function roleKey(user: { is_platform_admin: boolean; is_staff: boolean }): 'owner' | 'staff' | 'member' {
		return user.is_platform_admin ? 'owner' : user.is_staff ? 'staff' : 'member';
	}
	const ROLE_LABELS = { owner: m.users_owner, staff: m.users_staff, member: m.users_member };

	function handleSearch(query: string) {
		goto(`?query=${encodeURIComponent(query)}&page=1`, { keepFocus: true });
	}

	function handlePageChange(nextPage: number) {
		const params = new URLSearchParams({ page: String(nextPage) });
		if (data.query) params.set('query', data.query);
		goto(`?${params}`, { keepFocus: true });
	}

	let sheetOpen = $state(false);
	let sheetKind = $state<'user' | 'role' | null>(null);
	let activeUserId = $state<string | null>(null);
	let userDetail = $state<AdminUserDetail | null>(null);
	let detailLoading = $state(false);
	let detailError = $state<string | null>(null);

	let displayName = $state('');
	let username = $state('');

	let selectedResource = $state(ADMIN_PERMISSION_RESOURCES[0]?.resource ?? CUSTOM_PERMISSION_RESOURCE);
	let customResource = $state('');
	let actionView = $state(false);
	let actionManage = $state(false);

	async function openSheet(userId: string, kind: 'user' | 'role') {
		activeUserId = userId;
		sheetKind = kind;
		userDetail = null;
		detailError = null;
		detailLoading = true;
		sheetOpen = true;

		const body = new FormData();
		body.set('userId', userId);
		const response = await fetch('?/loadUserDetail', { method: 'POST', body });
		const result = deserialize(await response.text());
		detailLoading = false;

		if (result.type === 'success' && result.data?.user) {
			userDetail = result.data.user as AdminUserDetail;
			displayName = userDetail.display_name ?? '';
			username = userDetail.username ?? '';
			selectedResource = ADMIN_PERMISSION_RESOURCES[0]?.resource ?? CUSTOM_PERMISSION_RESOURCE;
			customResource = '';
			actionView = false;
			actionManage = false;
		} else {
			detailError = (result.type === 'failure' && (result.data?.error as string)) || m.users_load_failed();
		}
	}

	$effect(() => {
		if (!sheetOpen) {
			sheetKind = null;
			activeUserId = null;
			userDetail = null;
			detailError = null;
		}
	});
</script>

<PageHeader title={m.admin_users()} lede={m.users_lede()} agent="personality" />

<div>
	<DataTable
		card
		{columns}
		page={data.page}
		pageSize={data.pageSize}
		totalItems={data.total}
		onPageChange={handlePageChange}
		searchQuery={data.query}
		onSearch={handleSearch}
		searchPlaceholder={m.users_search()}
	>
		{#each data.users as user (user.id)}
			<tr>
				<td>
					<span class="person">
						<Avatar name={user.email} size={32} />
						<span class="person__email">{user.email}</span>
					</span>
				</td>
				<td><span class="role" data-role={roleKey(user)}>{ROLE_LABELS[roleKey(user)]()}</span></td>
				<td class="spaces">{user.org_count}</td>
				<td>
					<Menu>
						{#snippet trigger({ toggle })}
							<IconButton onclick={toggle} aria-label={m.users_actions_for({ email: user.email })}>
								<IconMore size={18} />
							</IconButton>
						{/snippet}
						<MenuItem onclick={() => openSheet(user.id, 'user')}>{m.users_update_user()}</MenuItem>
						<MenuItem onclick={() => openSheet(user.id, 'role')}>{m.users_update_role()}</MenuItem>
						{#if !user.is_staff}
							<form method="POST" action="?/promote" use:enhance>
								<input type="hidden" name="userId" value={user.id} />
								<MenuItem type="submit">{m.users_promote()}</MenuItem>
							</form>
						{/if}
					</Menu>
				</td>
			</tr>
		{/each}
	</DataTable>
	{#if data.users.length === 0}
		<p class="md-body-medium mt-4" style="color: var(--md-sys-color-on-surface-variant)">
			{data.query ? m.users_no_match({ query: data.query }) : m.users_none()}
		</p>
	{/if}
</div>

<BottomSheet bind:open={sheetOpen}>
	{#if detailLoading}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.common_loading()}</p>
	{:else if detailError}
		<p class="md-body-medium" style="color: var(--md-sys-color-error)">{detailError}</p>
	{:else if userDetail && sheetKind === 'user'}
		<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{m.users_update_user()}</h2>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{userDetail.email}</p>

		<form
			method="POST"
			action="?/updateUser"
			use:enhance={() => {
				return async ({ result, update }) => {
					await update({ reset: false });
					if (result.type === 'success' && result.data?.user) {
						userDetail = result.data.user as AdminUserDetail;
					}
				};
			}}
			class="mt-6 flex flex-col gap-3"
		>
			<input type="hidden" name="userId" value={activeUserId} />
			<TextField id="display_name" name="display_name" label={m.profile_display_name()} bind:value={displayName} />
			<TextField id="username" name="username" label={m.profile_username()} bind:value={username} />
			<Button type="submit" variant="filled" class="w-fit">{m.common_save()}</Button>
		</form>
	{:else if userDetail && sheetKind === 'role'}
		<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{m.users_update_role()}</h2>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{userDetail.email}</p>

		<section class="mt-6">
			<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.users_permissions()}</h3>
			{#if userDetail.permissions.length === 0}
				<p class="md-body-small mt-2" style="color: var(--md-sys-color-on-surface-variant)">{m.users_no_grants()}</p>
			{:else}
				<List class="mt-2">
					{#each userDetail.permissions as grant (grant.id)}
						<ListItem
							headline="{grant.scope_type === 'admin' ? m.admin_tag() : (grant.org_name ?? m.users_unknown_org())}: {grant.resource}"
							supportingText={grant.actions.join(', ')}
						>
							{#snippet trailing()}
								{#if data.canManageUsers}
									<form
										method="POST"
										action="?/revokePermission"
										use:enhance={() => {
											return async ({ result, update }) => {
												await update({ reset: false });
												if (result.type === 'success' && result.data?.permissions && userDetail) {
													userDetail = { ...userDetail, permissions: result.data.permissions as AdminUserDetail['permissions'] };
												}
											};
										}}
									>
										<input type="hidden" name="userId" value={activeUserId} />
										<input type="hidden" name="permissionId" value={grant.id} />
										<IconButton type="submit" aria-label={m.users_revoke({ resource: grant.resource })}>
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
				<form
					method="POST"
					action="?/grantPermission"
					use:enhance={() => {
						return async ({ result, update }) => {
							await update({ reset: false });
							if (result.type === 'success' && result.data?.permissions && userDetail) {
								userDetail = { ...userDetail, permissions: result.data.permissions as AdminUserDetail['permissions'] };
								actionView = false;
								actionManage = false;
							}
						};
					}}
					class="mt-4 flex flex-col gap-4"
				>
					<input type="hidden" name="userId" value={activeUserId} />
					<div>
						<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{m.users_resource()}</span>
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
								<Radio name="resource" value={CUSTOM_PERMISSION_RESOURCE} bind:group={selectedResource} label={m.users_other()} />
							</div>
							{#if selectedResource === CUSTOM_PERMISSION_RESOURCE}
								<TextField
									id="customResource"
									name="customResource"
									label={m.users_custom_resource()}
									bind:value={customResource}
									supportingText={m.users_custom_hint()}
								/>
							{/if}
						</div>
					</div>

					<div>
						<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{m.users_actions()}</span>
						<div class="mt-2 flex gap-4">
							<Checkbox name="actions" value="view" bind:checked={actionView} label={m.users_view()} />
							<Checkbox name="actions" value="manage" bind:checked={actionManage} label={m.users_manage()} />
						</div>
					</div>

					<Button type="submit" variant="filled" class="w-fit">{m.users_grant()}</Button>
				</form>
			{:else}
				<p class="md-body-small mt-4" style="color: var(--md-sys-color-on-surface-variant)">
					{m.users_need_manage()}
				</p>
			{/if}
		</section>
	{/if}
</BottomSheet>

<style>
	.person {
		display: inline-flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
	}
	.person__email {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.role {
		display: inline-flex;
		padding: 3px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 650;
	}
	.role[data-role='staff'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.role[data-role='owner'] {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.spaces {
		font-family: var(--md-ref-typeface-mono);
		font-variant-numeric: tabular-nums;
	}
	.m3-resource-row {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 0;
	}
</style>
