<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { monthLabel } from '$lib/money';
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
	import Select from '$lib/components/m3/Select.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import { formatShare, formatTokens, formatTokensFull, usageShare } from '$lib/usage';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminSubscription, AdminUserDetail } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const columns = [
		{ key: 'email', label: m.users_person() },
		{ key: 'staff', label: m.users_role() },
		{ key: 'orgs', label: m.users_spaces() },
		{ key: 'plan', label: m.users_plan() },
		{ key: 'usage', label: m.users_usage() },
		{ key: 'actions', label: '' },
	];

	function roleKey(user: { is_platform_admin: boolean; is_staff: boolean }): 'owner' | 'staff' | 'member' {
		return user.is_platform_admin ? 'owner' : user.is_staff ? 'staff' : 'member';
	}
	const ROLE_LABELS = { owner: m.users_owner, staff: m.users_staff, member: m.users_member };

	// Search, page and page size live in the URL.
	function show(next: { query?: string; page?: number; size?: number }) {
		const params = new URLSearchParams({ page: String(next.page ?? data.page) });
		const query = next.query ?? data.query;
		if (query) params.set('query', query);
		const size = next.size ?? data.pageSize;
		if (size !== 20) params.set('size', String(size));
		goto(`?${params}`, { keepFocus: true, noScroll: true });
	}
	const handleSearch = (query: string) => show({ query, page: 1 });
	const handlePageChange = (page: number) => show({ page });
	// A new page size starts from the page holding the first row shown now.
	const handlePageSizeChange = (size: number) => show({ size, page: Math.floor(((data.page - 1) * data.pageSize) / size) + 1 });

	let sheetOpen = $state(false);
	let sheetKind = $state<'user' | 'role' | 'plan' | null>(null);
	let subscription = $state<AdminSubscription | null>(null);
	let subPlan = $state('');
	let subQuota = $state('');
	let subUntil = $state('');
	let subNote = $state('');
	let subError = $state<string | null>(null);
	let subSaved = $state(false);
	const planOptions = $derived(data.plans.map((plan) => ({ value: plan.id, label: `${plan.name} · ${formatTokens(plan.monthly_tokens)}` })));

	function fillSubscription(loaded: AdminSubscription) {
		subscription = loaded;
		subPlan = loaded.plan.id;
		subQuota = loaded.quota_override?.toString() ?? '';
		subUntil = loaded.override_until ? loaded.override_until.slice(0, 10) : '';
		subNote = '';
	}

	async function openPlanSheet(userId: string) {
		activeUserId = userId;
		sheetKind = 'plan';
		subscription = null;
		subError = null;
		subSaved = false;
		detailError = null;
		detailLoading = true;
		sheetOpen = true;
		const body = new FormData();
		body.set('userId', userId);
		const result = deserialize(await (await fetch('?/loadSubscription', { method: 'POST', body })).text());
		detailLoading = false;
		if (result.type === 'success' && result.data?.subscription) fillSubscription(result.data.subscription as AdminSubscription);
		else detailError = (result.type === 'failure' && (result.data?.error as string)) || m.users_load_failed();
	}

	const shortDate = (at: string) => new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium' }).format(new Date(at));
	const activeEmail = $derived(data.users.find((u) => u.id === activeUserId)?.email ?? '');
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

{#snippet actions(user: (typeof data.users)[number])}
	<Menu>
		{#snippet trigger({ toggle })}
			<IconButton onclick={toggle} aria-label={m.users_actions_for({ email: user.email })}>
				<IconMore size={18} />
			</IconButton>
		{/snippet}
		<MenuItem onclick={() => openSheet(user.id, 'user')}>{m.users_update_user()}</MenuItem>
		<MenuItem onclick={() => openSheet(user.id, 'role')}>{m.users_update_role()}</MenuItem>
		<MenuItem onclick={() => openPlanSheet(user.id)}>{m.users_plan_quota()}</MenuItem>
		{#if !user.is_staff}
			<form method="POST" action="?/promote" use:enhance>
				<input type="hidden" name="userId" value={user.id} />
				<MenuItem type="submit">{m.users_promote()}</MenuItem>
			</form>
		{/if}
	</Menu>
{/snippet}

<div>
	<DataTable
		card
		{columns}
		page={data.page}
		pageSize={data.pageSize}
		totalItems={data.total}
		onPageChange={handlePageChange}
		pageSizeOptions={data.pageSizes}
		onPageSizeChange={handlePageSizeChange}
		searchQuery={data.query}
		onSearch={handleSearch}
		searchPlaceholder={m.users_search()}
	>
		{#each data.users as user (user.id)}
			<tr>
				<td>
					<span class="person">
						<Avatar name={user.email} size={32} />
						<span class="person__email" title={user.email}>{user.email}</span>
					</span>
				</td>
				<td><span class="role" data-role={roleKey(user)}>{ROLE_LABELS[roleKey(user)]()}</span></td>
				<td class="spaces">{user.org_count}</td>
				<td>
					<span class="plan-name">{user.plan_name}</span>
					{#if user.custom_quota}<span class="custom-chip">{m.users_custom_quota()}</span>{/if}
				</td>
				<td class="usage-cell">
					<WavyProgress value={usageShare(user.tokens_used, user.monthly_tokens)} tone={user.tokens_used >= user.monthly_tokens ? 'ember' : 'glow'} label={formatShare(usageShare(user.tokens_used, user.monthly_tokens))} />
					<span class="usage-text">{m.usage_of_tokens({ used: formatTokens(user.tokens_used), total: formatTokens(user.monthly_tokens) })}</span>
				</td>
				<td>{@render actions(user)}</td>
			</tr>
		{/each}
		{#snippet list()}
			{#each data.users as user (user.id)}
				<div role="listitem" class="user-item">
					<Avatar name={user.email} size={40} />
					<div class="user-item__text">
						<span class="user-item__email">{user.email}</span>
						<span class="user-item__meta">
							<span class="role" data-role={roleKey(user)}>{ROLE_LABELS[roleKey(user)]()}</span>
							<span>{user.plan_name}</span>
							{#if user.custom_quota}<span class="custom-chip">{m.users_custom_quota()}</span>{/if}
							<span>· {user.org_count === 1 ? m.users_spaces_one() : m.users_spaces_count({ count: user.org_count })}</span>
						</span>
						<span class="user-item__usage">
							<WavyProgress value={usageShare(user.tokens_used, user.monthly_tokens)} tone={user.tokens_used >= user.monthly_tokens ? 'ember' : 'glow'} label={formatShare(usageShare(user.tokens_used, user.monthly_tokens))} />
							<span class="usage-text">{m.usage_of_tokens({ used: formatTokens(user.tokens_used), total: formatTokens(user.monthly_tokens) })}</span>
						</span>
					</div>
					{@render actions(user)}
				</div>
			{/each}
		{/snippet}
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
	{:else if subscription && sheetKind === 'plan'}
		<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{m.users_plan_quota()}</h2>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{activeEmail}</p>
		<div class="sub-usage mt-4">
			<span class="sub-usage__label">{m.sub_this_month({ month: monthLabel(subscription.month) })}</span>
			<WavyProgress value={usageShare(subscription.tokens_used, subscription.monthly_tokens)} tone={subscription.tokens_used >= subscription.monthly_tokens ? 'ember' : 'glow'} label={formatShare(usageShare(subscription.tokens_used, subscription.monthly_tokens))} />
			<span class="sub-usage__numbers">{m.sub_used_of({ used: formatTokensFull(subscription.tokens_used), total: formatTokensFull(subscription.monthly_tokens) })}</span>
		</div>
		{#if data.canManageUsers}
			<form
				method="POST"
				action="?/saveSubscription"
				class="mt-6 flex flex-col gap-3"
				use:enhance={() => {
					subError = null;
					subSaved = false;
					return async ({ result, update }) => {
						await update({ reset: false });
						if (result.type === 'success' && result.data?.subscription) {
							fillSubscription(result.data.subscription as AdminSubscription);
							subSaved = true;
						} else if (result.type === 'failure') {
							subError = (result.data?.error as string) ?? m.sub_save_failed();
						}
					};
				}}
			>
				<input type="hidden" name="userId" value={activeUserId} />
				<Select name="plan_id" label={m.users_plan()} options={planOptions} bind:value={subPlan} />
				<TextField id="sub-quota" name="quota_override" type="number" min="0" step="1" label={m.sub_quota()} bind:value={subQuota} supportingText={m.sub_quota_hint()} />
				{#if subQuota.trim() !== ''}
					<TextField id="sub-until" name="override_until" type="date" label={m.sub_until()} bind:value={subUntil} supportingText={m.sub_until_hint()} />
				{/if}
				<TextField id="sub-note" name="note" label={m.sub_note()} bind:value={subNote} supportingText={m.sub_note_hint()} />
				{#if subError}<p class="md-body-medium" style="color: var(--md-sys-color-error)" role="alert">{subError}</p>{/if}
				{#if subSaved}<p class="md-body-medium" style="color: var(--md-sys-color-primary)" role="status">{m.sub_saved()}</p>{/if}
				<Button type="submit" variant="filled" class="w-fit">{m.sub_save()}</Button>
			</form>
		{/if}
		{#if subscription.history.length > 0}
			<section class="mt-6">
				<h3 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{m.sub_history()}</h3>
				<List class="mt-2">
					{#each subscription.history as change (change.created_at)}
						<ListItem
							headline={change.quota_override !== null ? `${change.plan_name} · ${formatTokensFull(change.quota_override)}` : change.plan_name}
							supportingText={[shortDate(change.created_at), change.changed_by_email, change.note].filter(Boolean).join(' · ')}
						/>
					{/each}
				</List>
			</section>
		{/if}
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
	/* Phone: each person is a list item. */
	.user-item {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 12px 8px 12px 12px;
		border-radius: var(--md-sys-shape-corner-large);
	}
	.user-item + .user-item {
		border-top: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0;
	}
	.user-item__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.user-item__email {
		overflow: hidden;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.user-item__meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.user-item__usage {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-width: 260px;
	}

	.person {
		display: inline-flex;
		align-items: center;
		gap: 12px;
		min-width: 0;
	}
	.plan-name {
		font-weight: 600;
	}
	.custom-chip {
		margin-left: 6px;
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-size: 0.6875rem;
		font-weight: 700;
	}
	.usage-cell {
		min-width: 140px;
	}
	.usage-text,
	.sub-usage__label,
	.sub-usage__numbers {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sub-usage {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	/* One line; a very long address ends in "…" (its full text is the title). */
	.person__email {
		display: block;
		max-width: 28ch;
		overflow: hidden;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
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
