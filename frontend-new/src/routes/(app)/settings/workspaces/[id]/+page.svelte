<script lang="ts">
	import { Folder, Lock, LogOut, Mail, UserMinus, Users } from '@lucide/svelte';
	import { invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { ROLE_NAME } from '$lib/settings/workspaces';
	import { canLeave, inviteRows, isManager, memberRows, type MemberRow } from '$lib/settings/workspace';
	import { workspaceRows } from '$lib/adapters/shell';
	import type { OrgRole } from '$lib/adapters/api';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import WorkspaceTile from '$lib/ds/settings/WorkspaceTile.svelte';
	import WorkspaceStats from '$lib/ds/settings/WorkspaceStats.svelte';
	import InviteBox from '$lib/ds/settings/InviteBox.svelte';
	import PersonRow from '$lib/ds/settings/PersonRow.svelte';
	import WorkspaceBoundary from '$lib/ds/settings/WorkspaceBoundary.svelte';

	// Settings › Workspace (App / Settings / Workspace wZm9B): one workspace. Its name (renamed by
	// owners and admins), its numbers, inviting people (as members or admins), the members with
	// their roles where the person may change them and Remove, the invites still open (Copy link,
	// Revoke), leaving, and who can hand it over; beside it, what's shared there and what stays
	// personal. Left out (AGENTS.md §9, BACKEND_TODO.md): who made it and when, shared templates,
	// each member's projects and activity, Resend, and deleting the workspace.
	let { data } = $props();

	const detail = $derived(data.detail);
	const path = $derived(`/settings/workspaces/${detail.id}`);
	const manager = $derived(isManager(detail.my_role));
	const now = new Date();
	const members = $derived(memberRows(detail, now));
	const invites = $derived(inviteRows(detail.invites, now));
	const tone = $derived(data.summary ? (workspaceRows({ orgs: [data.summary], project_org_id: null })[0]?.tone ?? 'plain') : 'plain');
	const initial = $derived([...detail.name.trim()][0]?.toUpperCase() ?? '?');
	const roleName = (role: OrgRole) => ROLE_NAME[role]();

	async function act(name: string, fields: Record<string, string>, done: string) {
		const result = await pageAction(path, name, fields);
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return false;
		}
		toasts.show({ kind: 'success', title: done });
		await invalidate('app:workspace');
		return true;
	}

	// Rename.
	let renaming = $state(false);
	let name = $state('');
	let nameError = $state<string>();
	let saving = $state(false);
	function startRenaming() {
		name = detail.name;
		nameError = undefined;
		renaming = true;
	}
	async function rename() {
		saving = true;
		const result = await pageAction(path, 'rename', { name });
		saving = false;
		if (!result.ok) return void (nameError = result.message || m.action_failed());
		renaming = false;
		toasts.show({ kind: 'success', title: m.workspace_renamed({ name: name.trim() }) });
		await invalidate('app:workspace');
		await invalidate('app:shell');
	}

	// Invite.
	let box = $state<InviteBox>();
	let inviting = $state(false);
	let inviteError = $state<string>();
	async function invite(emails: string[], role: string) {
		inviting = true;
		inviteError = undefined;
		const result = await pageAction(path, 'invite', { emails: emails.join(','), role });
		inviting = false;
		if (!result.ok) return void (inviteError = result.message || m.action_failed());
		const { sent, failed } = result.data as { sent: number; failed: { email: string; message: string }[] };
		if (sent) toasts.show({ kind: 'success', title: sent === 1 ? m.workspace_invited_one() : m.workspace_invited_many({ count: sent }) });
		if (failed.length) inviteError = failed.map((one) => `${one.email}: ${one.message}`).join(' · ');
		else box?.clear();
		await invalidate('app:workspace');
	}

	// Remove someone, after asking.
	let removing = $state<MemberRow | null>(null);
	let busy = $state(false);
	async function remove() {
		if (!removing) return;
		busy = true;
		const gone = await act('remove', { user: removing.id }, m.workspace_removed({ name: removing.name }));
		busy = false;
		if (gone) removing = null;
	}

	async function copyLink(code: string) {
		const link = `${page.url.origin}/invite/${encodeURIComponent(code)}`;
		const copied = await navigator.clipboard?.writeText(link).then(() => true, () => false);
		toasts.show(copied ? { kind: 'success', title: m.workspace_link_copied() } : { kind: 'error', title: m.workspace_link_copy_failed(), detail: link });
	}

	let leaving = $state(false);
</script>

<svelte:head><title>{detail.name} · {m.app_name()}</title></svelte:head>

<div class="h-full overflow-y-auto">
	<div class="flex max-w-316 flex-col gap-14 px-4 pt-8 pb-12 lg:flex-row lg:px-12">
		<div class="flex min-w-0 flex-1 flex-col gap-8 lg:max-w-220">
			<header class="flex flex-wrap items-center gap-4">
				<WorkspaceTile {initial} {tone} size={56} />
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<h1 class="truncate text-display font-medium tracking-display text-text">{detail.name}</h1>
					<p class="font-mono text-label text-text-3">{m.workspace_you_are({ role: roleName(detail.my_role) })}</p>
				</div>
				{#if manager}<Button variant="secondary" label={m.workspace_rename()} onclick={startRenaming} />{/if}
			</header>

			<WorkspaceStats
				label={m.workspace_in_numbers()}
				stats={[
					...(data.summary ? [{ icon: Folder, value: data.summary.projects, label: m.workspace_stat_projects() }] : []),
					{ icon: Users, value: detail.members.length, label: m.workspace_stat_members() },
					...(manager ? [{ icon: Mail, value: detail.invites.length, label: m.workspace_stat_invites() }] : []),
				]}
			/>

			{#if manager}
				<section aria-labelledby="invite-title" class="flex flex-col gap-3">
					<header class="flex flex-col gap-0.5">
						<h2 id="invite-title" class="text-body font-medium text-text">{m.workspace_invite_title()}</h2>
						<p class="text-caption text-text-3">{m.workspace_invite_body()}</p>
					</header>
					<InviteBox
						bind:this={box}
						labels={{
							field: m.workspace_invite_field(),
							placeholder: m.workspace_invite_placeholder(),
							remove: m.workspace_invite_remove({ email: '{email}' }),
							role: m.workspace_invite_role(),
							note: m.workspace_invite_note(),
							send: (count) => (count === 0 ? m.workspace_invite_send_none() : count === 1 ? m.workspace_invite_send_one() : m.workspace_invite_send({ count })),
						}}
						roles={[
							{ value: 'member', title: m.workspace_role_member(), body: m.workspace_role_member_body() },
							{ value: 'admin', title: m.workspace_role_admin(), body: m.workspace_role_admin_body() },
						]}
						error={inviteError}
						busy={inviting}
						onsend={invite}
					/>
				</section>
			{/if}

			<section aria-labelledby="members-title" class="flex flex-col gap-3">
				<h2 id="members-title" class="text-body font-medium text-text">{m.workspace_members_title()}</h2>
				<ul class="flex flex-col border-t border-border">
					{#each members as member (member.id)}
						<PersonRow name={member.name} initial={[...member.name][0]?.toUpperCase() ?? '?'} avatar={member.avatar} you={member.you ? m.workspace_you() : undefined} meta={member.meta}>
							{#snippet controls()}
								{#if member.roleOptions.length}
									<div class="w-32">
										<Select
											label={m.workspace_role_of({ name: member.name })}
											options={member.roleOptions.map((role) => ({ value: role, label: roleName(role) }))}
											value={member.role}
											onchange={(role) => act('role', { user: member.id, role }, m.workspace_role_changed({ name: member.name, role: roleName(role as OrgRole) }))}
										/>
									</div>
								{:else}
									<span class="text-caption text-text-2">{roleName(member.role)}</span>
								{/if}
								{#if member.canRemove}
									<IconButton variant="ghost" icon={UserMinus} label={m.workspace_remove_named({ name: member.name })} onclick={() => (removing = member)} />
								{/if}
							{/snippet}
						</PersonRow>
					{/each}
					{#each invites as invite (invite.code)}
						<PersonRow name={invite.email} initial={invite.email.charAt(0).toUpperCase()} meta={invite.meta}>
							{#snippet controls()}
								<span class="font-mono text-label text-warning">{m.workspace_pending({ role: roleName(invite.role) })}</span>
								<button type="button" onclick={() => copyLink(invite.code)} class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text">{m.workspace_copy_link()}</button>
								<button
									type="button"
									aria-label={m.workspace_revoke_named({ email: invite.email })}
									onclick={() => act('revoke', { code: invite.code }, m.workspace_revoked({ email: invite.email }))}
									class="cursor-pointer rounded-xs text-caption text-text-3 outline-offset-2 hover:text-text">{m.workspace_revoke()}</button
								>
							{/snippet}
						</PersonRow>
					{/each}
				</ul>
			</section>

			<section aria-labelledby="leave-title" class="flex flex-col gap-3">
				<h2 id="leave-title" class="text-body font-medium text-text">{m.workspace_leave_title()}</h2>
				<div class="flex flex-col border-t border-border">
					<SettingRow title={m.workspace_leave({ name: detail.name })} description={canLeave(detail) ? m.workspace_leave_body() : m.workspace_leave_last_owner()}>
						{#snippet control()}
							<Button variant="danger" iconStart={LogOut} label={m.workspace_leave_action()} disabled={!canLeave(detail)} onclick={() => (leaving = true)} />
						{/snippet}
					</SettingRow>
					<SettingRow title={m.workspace_ownership()} description={detail.my_role === 'owner' ? m.workspace_ownership_owner() : m.workspace_ownership_other()}>
						{#snippet control()}
							{#if detail.my_role !== 'owner'}<span class="inline-flex text-text-3"><Icon icon={Lock} size={14} /></span>{/if}
						{/snippet}
					</SettingRow>
				</div>
			</section>
		</div>

		<div class="w-full shrink-0 lg:w-58">
			<WorkspaceBoundary
				shared={{ title: m.workspace_shared_here(), items: [m.workspace_shared_projects(), m.workspace_shared_builds()] }}
				personal={{ title: m.workspace_personal_title(), items: [m.workspace_personal_chats(), m.workspace_personal_memory(), m.workspace_personal_inbox()] }}
				tip={m.workspace_switch_tip()}
			/>
		</div>
	</div>
</div>

<Dialog bind:open={renaming} title={m.workspace_rename_title()}>
	<form
		class="pt-2"
		onsubmit={(event) => {
			event.preventDefault();
			void rename();
		}}
	>
		<TextField label={m.workspace_name()} bind:value={name} error={nameError} required />
	</form>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (renaming = false)} />
		<Button variant="inverse" label={m.workspace_rename_save()} loading={saving} onclick={rename} />
	{/snippet}
</Dialog>

<Dialog bind:open={() => removing !== null, (open) => !open && (removing = null)} title={m.workspace_remove_title({ name: removing?.name ?? '' })} description={m.workspace_remove_body()}>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (removing = null)} />
		<Button variant="danger" label={m.workspace_remove()} loading={busy} onclick={remove} />
	{/snippet}
</Dialog>

<form method="POST" action="?/leave">
	<Dialog bind:open={leaving} title={m.workspace_leave({ name: detail.name })} description={m.workspace_leave_body()}>
		{#snippet actions()}
			<Button variant="secondary" label={m.action_cancel()} onclick={() => (leaving = false)} />
			<Button type="submit" variant="danger" label={m.workspace_leave_action()} />
		{/snippet}
	</Dialog>
</form>
