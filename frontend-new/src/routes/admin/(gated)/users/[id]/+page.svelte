<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { accessFromGrants, can } from '$lib/admin/access';
	import { areaLabel, areaRows, levelLabels } from '$lib/admin/labels';
	import { compact, dateOnly, dateTime, full, localParts, monthLabel } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import AccessEditor from '$lib/ds/admin/AccessEditor.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Users & access → a person (user: view): their profile (write), their admin role area by area,
	// up to the editor's own level (manage), the workspaces they're in and their role there (write),
	// their plan and an allowance of their own (plans: view, changed with plans: write), and grants
	// given in a workspace (revoked with manage). A part the viewer can't change is shown as it is.
	let { data, form } = $props();

	const language = getLocale();
	const ROLES = [
		{ value: 'member', label: m.admin_role_member() },
		{ value: 'admin', label: m.admin_role_admin() },
		{ value: 'owner', label: m.admin_role_owner() },
	];
	const roleLabel = (role: string) => ROLES.find((option) => option.value === role)?.label ?? role;

	const canWrite = $derived(can(data.access, 'user', 'write'));
	const canManage = $derived(can(data.access, 'user', 'manage'));
	const canPlan = $derived(can(data.access, 'plans', 'write'));

	let revoking = $state<{ id: string; resource: string; workspace: string } | null>(null);
	let revokeForm: HTMLFormElement | undefined = $state();
	let confirmOpen = $state(false);
</script>

{#snippet save()}<Button label={m.admin_save()} variant="inverse" type="submit" />{/snippet}

{#if data.person.ok}
	{@const person = data.person.data}
	{@const grants = person.permissions.filter((grant) => grant.scope_type !== 'admin')}
	{@const joinable = (data.orgs?.ok ? data.orgs.data : []).filter((org) => !person.memberships.some((membership) => membership.org_id === org.id))}

	<PageHeader title={person.email} meta={person.id} back={{ href: '/admin/users', label: m.admin_user_back() }} />

	<form method="POST" action="?/profile" use:enhance={() => ({ update }) => update({ reset: false })}>
	<FormSection title={m.admin_user_profile()} description={m.admin_user_profile_desc()} footer={canWrite ? save : undefined}>
		{#if canWrite}
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField label={m.admin_user_name()} name="display_name" value={person.display_name ?? ''} autocomplete="off" />
				<TextField label={m.admin_user_username()} name="username" value={person.username ?? ''} mono autocomplete="off" />
			</div>
		{:else}
			<dl class="grid gap-4 text-ui sm:grid-cols-2">
				<div class="flex flex-col gap-1"><dt class="text-text-3">{m.admin_user_name()}</dt><dd class="text-text">{person.display_name ?? '—'}</dd></div>
				<div class="flex flex-col gap-1"><dt class="text-text-3">{m.admin_user_username()}</dt><dd class="font-mono text-text">{person.username ?? '—'}</dd></div>
			</dl>
		{/if}
		<Outcome {form} action="profile" done={m.admin_user_profile_saved()} failed={m.admin_user_profile_failed()} />
	</FormSection>
	</form>

	<form method="POST" action="?/access" use:enhance={() => ({ update }) => update({ reset: false })}>
	<FormSection title={m.admin_user_access()} description={m.admin_user_access_desc()} footer={canManage && !person.is_platform_admin ? save : undefined}>
		{#if person.is_platform_admin}
			<Notice kind="info" text={m.admin_user_access_platform()} />
		{:else}
				<AccessEditor
					label={m.admin_user_access_label({ email: person.email })}
					areas={areaRows()}
					value={accessFromGrants(person.permissions)}
					max={data.access}
					levelLabels={levelLabels()}
					lockedNote={m.admin_user_access_locked()}
					readonly={!canManage}
					name="access"
				/>
			<Outcome {form} action="access" done={m.admin_user_access_saved()} failed={m.admin_user_access_failed()} />
		{/if}
	</FormSection>
	</form>

	<FormSection title={m.admin_user_workspaces()} description={m.admin_user_workspaces_desc()}>
		{#if person.memberships.length === 0}
			<p class="text-ui text-text-3">{m.admin_user_workspaces_empty()}</p>
		{:else}
			<ul class="flex flex-col">
				{#each person.memberships as membership (membership.org_id)}
					<li class="flex items-center gap-3 border-b border-border py-2.5 last:border-b-0">
						<span class="min-w-0 flex-1 truncate text-ui text-text">{membership.org_name}</span>
						<span class="font-mono text-label text-text-3">{roleLabel(membership.role)}</span>
						{#if canWrite}
							<form method="POST" action="?/removeMembership" use:enhance>
								<input type="hidden" name="org_id" value={membership.org_id} />
								<Button label={m.admin_user_remove_workspace({ workspace: membership.org_name })} variant="ghost" type="submit" />
							</form>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
		{#if canWrite && joinable.length > 0}
			<form method="POST" action="?/addMembership" use:enhance class="flex flex-wrap items-end gap-3">
				<div class="min-w-48 flex-1">
					<Select label={m.admin_user_workspace()} name="org_id" options={joinable.map((org) => ({ value: org.id, label: org.name }))} />
				</div>
				<div class="w-40"><Select label={m.admin_user_role()} name="role" options={ROLES} /></div>
				<Button label={m.admin_user_add_workspace()} variant="secondary" type="submit" />
			</form>
		{/if}
		<Outcome {form} action="addMembership" done={m.admin_user_workspace_added()} failed={m.admin_user_workspace_failed()} />
		<Outcome {form} action="removeMembership" done={m.admin_user_workspace_removed()} failed={m.admin_user_workspace_failed()} />
	</FormSection>

	{#if data.subscription}
		<form method="POST" action="?/subscription" use:enhance={() => ({ update }) => update({ reset: false })}>
		<FormSection title={m.admin_user_plan()} description={m.admin_user_plan_desc()} footer={canPlan && data.plans?.ok && data.subscription.ok ? save : undefined}>
			{#if data.subscription.ok}
				{@const subscription = data.subscription.data}
				{@const until = localParts(subscription.override_until, 'UTC').date}
				<p class="font-mono text-label text-text-2">
					{m.admin_user_used({ used: compact(subscription.tokens_used, language), total: compact(subscription.monthly_tokens, language), month: monthLabel(subscription.month, language) })}
				</p>
				{#if subscription.quota_override !== null && subscription.override_until}
					<p class="text-caption text-text-3">{m.admin_user_own_until({ date: dateOnly(subscription.override_until, language, 'UTC') })}</p>
				{/if}
				{#if canPlan && data.plans?.ok}
					<div class="grid gap-4 sm:grid-cols-2">
						<Select label={m.admin_user_plan_field()} name="plan_id" value={subscription.plan.id} options={data.plans.data.map((plan) => ({ value: plan.id, label: plan.name }))} />
						<TextField
							label={m.admin_user_quota()}
							name="quota_override"
							value={subscription.quota_override === null ? '' : String(subscription.quota_override)}
							helper={m.admin_user_quota_helper({ tokens: full(subscription.plan.monthly_tokens, language) })}
							mono
							autocomplete="off"
						/>
						<TextField label={m.admin_user_until()} name="override_until" value={until} helper={m.admin_user_until_helper()} placeholder="2026-12-31" mono autocomplete="off" />
						<div class="sm:col-span-2"><TextArea label={m.admin_user_note()} name="note" value={subscription.note ?? ''} helper={m.admin_user_note_helper()} /></div>
					</div>
				{:else}
					<p class="text-ui text-text">{subscription.plan.name}</p>
				{/if}
				<Outcome {form} action="subscription" done={m.admin_user_plan_saved()} failed={m.admin_user_plan_failed()} />
				{#if subscription.history.length > 0}
					<section aria-label={m.admin_user_history()} class="flex flex-col gap-2">
						<SectionLabel label={m.admin_user_history()} />
						<ol class="flex flex-col gap-1.5">
							{#each subscription.history as change, index (index)}
								<li class="flex flex-wrap items-baseline gap-x-2 text-caption">
									<span class="font-mono text-text-3">{dateTime(change.created_at, language)}</span>
									<span class="text-text">{m.admin_user_history_entry({ plan: change.plan_name, tokens: change.quota_override === null ? '—' : compact(change.quota_override, language) })}</span>
									{#if change.changed_by_email}<span class="text-text-3">{m.admin_user_history_by({ email: change.changed_by_email })}</span>{/if}
									{#if change.note}<span class="text-text-2">· {change.note}</span>{/if}
								</li>
							{/each}
						</ol>
					</section>
				{/if}
			{:else}
				<LoadFailed title={m.admin_user_plan_load_failed()} problem={data.subscription.problem} />
			{/if}
		</FormSection>
		</form>
	{/if}

	{#if grants.length > 0}
		<FormSection title={m.admin_user_grants()} description={m.admin_user_grants_desc()}>
			<ul class="flex flex-col">
				{#each grants as grant (grant.id)}
					{@const workspace = grant.org_name ?? m.admin_user_workspace_any()}
					<li class="flex items-center gap-3 border-b border-border py-2.5 last:border-b-0">
						<span class="min-w-0 flex-1 text-ui text-text">{workspace}</span>
						<span class="font-mono text-label text-text-2">{grant.resource}: {grant.actions.join(', ')}</span>
						{#if canManage}
							<Button
								label={m.admin_user_revoke()}
								variant="ghost"
								onclick={() => {
									revoking = { id: grant.id, resource: grant.resource, workspace };
									confirmOpen = true;
								}}
							/>
						{/if}
					</li>
				{/each}
			</ul>
			<Outcome {form} action="revoke" done={m.admin_user_revoked()} failed={m.admin_user_revoke_failed()} />
		</FormSection>
		<form bind:this={revokeForm} method="POST" action="?/revoke" use:enhance hidden>
			<input type="hidden" name="permission_id" value={revoking?.id ?? ''} />
		</form>
		<ConfirmDialog
			bind:open={confirmOpen}
			title={m.admin_user_revoke_title({ resource: revoking?.resource ?? '', workspace: revoking?.workspace ?? '' })}
			body={m.admin_user_revoke_body()}
			confirmLabel={m.admin_user_revoke()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				confirmOpen = false;
				revokeForm?.requestSubmit();
			}}
		/>
	{/if}
{:else}
	<PageHeader title={m.admin_page_users()} back={{ href: '/admin/users', label: m.admin_user_back() }} />
	<LoadFailed title={m.admin_user_failed()} problem={data.person.problem} />
{/if}
