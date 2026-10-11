<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconArrowBack from '$lib/components/icons/IconArrowBack.svelte';
	import IconMore from '$lib/components/icons/IconMore.svelte';
	import IconCopy from '$lib/components/icons/IconCopy.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import type { OrgMember, OrgRole } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const org = $derived(data.org);
	const manager = $derived(org.my_role === 'owner' || org.my_role === 'admin');

	const ROLE: Record<OrgRole, () => string> = {
		owner: m.org_role_owner,
		admin: m.org_role_admin,
		member: m.org_role_member,
	};
	const ROLE_HINT: Record<OrgRole, () => string> = {
		owner: m.org_role_owner_hint,
		admin: m.org_role_admin_hint,
		member: m.org_role_member_hint,
	};

	let renaming = $state(false);
	let name = $state('');
	let email = $state('');
	let inviteRole = $state<'member' | 'admin'>('member');
	// Which member's actions menu is open.
	let menus = $state<Record<string, boolean>>({});
	let status = $state('');

	// A confirmation for removing someone or leaving.
	let confirming = $state<{ kind: 'remove'; member: OrgMember } | { kind: 'leave' } | null>(null);
	let confirmOpen = $state(false);

	const error = $derived(form && 'error' in form ? (form.error as string) : null);
	const inviteError = $derived(form && 'inviteError' in form ? (form.inviteError as string) : null);

	$effect(() => {
		if (form && 'invited' in form && form.invited) {
			status = m.org_invite_sent({ email: form.invited as string });
			email = '';
		}
	});

	/** Roles this person may give `member`, other than the one they have. */
	function rolesFor(member: OrgMember): OrgRole[] {
		const all: OrgRole[] = ['owner', 'admin', 'member'];
		return all.filter((role) => {
			if (role === member.role) return false;
			if (org.my_role === 'owner') return true;
			// Admins move people between member and admin, and leave owners alone.
			return member.role !== 'owner' && role !== 'owner';
		});
	}

	function canRemove(member: OrgMember): boolean {
		if (member.user_id === org.me) return false;
		return org.my_role === 'owner' || (org.my_role === 'admin' && member.role !== 'owner');
	}

	function date(at: string): string {
		return new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium' }).format(new Date(at));
	}

	async function copyLink(code: string) {
		try {
			await navigator.clipboard.writeText(`${location.origin}/invite/${code}`);
			status = m.org_invite_copied();
		} catch {
			status = `${location.origin}/invite/${code}`;
		}
	}

	function confirmRemove(member: OrgMember) {
		menus[member.user_id] = false;
		confirming = { kind: 'remove', member };
		confirmOpen = true;
	}

	function confirmLeave() {
		confirming = { kind: 'leave' };
		confirmOpen = true;
	}

	const closeConfirm = () => async ({ update }: { update: () => Promise<void> }) => {
		await update();
		confirmOpen = false;
	};
</script>

<div class="org">
	<div class="org__inner">
		<a class="back" href="/organizations"><IconArrowBack size={18} />{m.org_back()}</a>

		<header class="head">
			<span class="head__mark" aria-hidden="true">{org.name.trim().charAt(0).toUpperCase() || '?'}</span>
			{#if renaming}
				<form
					method="POST"
					action="?/rename"
					class="rename"
					use:enhance={() => async ({ update, result }) => {
						await update({ reset: false });
						if (result.type === 'success') renaming = false;
					}}
				>
					<TextField name="name" label={m.org_name()} bind:value={name} maxlength={80} required class="rename__field" />
					<Button type="submit" variant="filled" size="s" disabled={!name.trim()}>{m.org_save()}</Button>
					<Button type="button" variant="text" size="s" onclick={() => (renaming = false)}>{m.org_cancel()}</Button>
				</form>
			{:else}
				<div class="head__text">
					<h1 class="md-headline-medium head__title">{org.name}</h1>
					<p class="head__meta">{ROLE[org.my_role]()} · {org.members.length === 1 ? m.org_pick_member() : m.org_pick_members({ count: org.members.length })}</p>
				</div>
				{#if manager}
					<Button variant="text" size="s" onclick={() => { name = org.name; renaming = true; }}>{m.org_rename()}</Button>
				{/if}
			{/if}
		</header>

		{#if error}<p class="error" role="alert">{error}</p>{/if}
		<p class="status" role="status" aria-live="polite">{status}</p>

		{#if manager}
			<section class="card" aria-labelledby="invite-title">
				<h2 id="invite-title" class="section-title">{m.org_invite_title()}</h2>
				<p class="section-lede">{m.org_invite_lede()}</p>
				<form method="POST" action="?/invite" class="invite" use:enhance={() => async ({ update }) => update({ reset: false })}>
					<TextField name="email" type="email" label={m.org_invite_email()} bind:value={email} autocomplete="off" required class="invite__email" error={!!inviteError} supportingText={inviteError ?? undefined} />
					<fieldset class="roles">
						<legend class="sr-only">{m.org_invite_role()}</legend>
						{#each ['member', 'admin'] as const as role (role)}
							<label class="roles__option" class:roles__option--on={inviteRole === role}>
								<input type="radio" name="role" value={role} bind:group={inviteRole} />
								<span class="roles__name">{ROLE[role]()}</span>
								<span class="roles__hint">{ROLE_HINT[role]()}</span>
							</label>
						{/each}
					</fieldset>
					<Button type="submit" variant="filled" size="m" disabled={!email.trim()}>{m.org_invite_send()}</Button>
				</form>

				{#if org.invites.length > 0}
					<h3 class="subhead">{m.org_invites_open()}</h3>
					<ul class="rows">
						{#each org.invites as invite (invite.code)}
							<li class="invite-row">
								<span class="invite-row__text">
									<span class="invite-row__email">{invite.email}</span>
									<span class="invite-row__meta">{ROLE[invite.role]()} · {m.org_invite_expires({ date: date(invite.expires_at) })}</span>
								</span>
								<IconButton type="button" aria-label={m.org_invite_copy()} title={m.org_invite_copy()} onclick={() => copyLink(invite.code)}>
									<IconCopy size={20} />
								</IconButton>
								<form method="POST" action="?/revoke" use:enhance>
									<input type="hidden" name="code" value={invite.code} />
									<IconButton type="submit" aria-label={m.org_invite_revoke()} title={m.org_invite_revoke()}>
										<IconClose size={20} />
									</IconButton>
								</form>
							</li>
						{/each}
					</ul>
				{/if}
			</section>
		{:else}
			<p class="section-lede">{m.org_only_managers()}</p>
		{/if}

		<section aria-labelledby="members-title">
			<h2 id="members-title" class="section-title">{m.org_members()}</h2>
			<ul class="rows rows--members">
				{#each org.members as member (member.user_id)}
					{@const roles = manager ? rolesFor(member) : []}
					<li class="member">
						<Avatar name={member.name} avatarUrl={member.avatar_url} size={40} />
						<span class="member__text">
							<span class="member__name">
								{member.name}
								{#if member.user_id === org.me}<span class="member__you">{m.org_you()}</span>{/if}
							</span>
							{#if member.email}<span class="member__email">{member.email}</span>{/if}
						</span>
						<span class="member__role">{ROLE[member.role]()}</span>
						{#if roles.length > 0 || canRemove(member)}
							<Menu bind:open={() => menus[member.user_id] ?? false, (open) => (menus[member.user_id] = open)}>
								{#snippet trigger({ toggle })}
									<IconButton type="button" aria-label={m.org_member_actions({ name: member.name })} onclick={toggle}>
										<IconMore size={20} />
									</IconButton>
								{/snippet}
								{#each roles as role (role)}
									<form method="POST" action="?/role" use:enhance={() => { menus[member.user_id] = false; return async ({ update }) => update(); }}>
										<input type="hidden" name="user_id" value={member.user_id} />
										<input type="hidden" name="role" value={role} />
										<MenuItem type="submit">{m.org_make({ role: ROLE[role]().toLowerCase() })}</MenuItem>
									</form>
								{/each}
								{#if canRemove(member)}
									<MenuItem onclick={() => confirmRemove(member)}>{m.org_remove()}</MenuItem>
								{/if}
							</Menu>
						{/if}
					</li>
				{/each}
			</ul>
		</section>

		<div class="leave">
			<Button variant="outlined" size="s" onclick={confirmLeave}>{m.org_leave()}</Button>
		</div>
	</div>
</div>

<Dialog bind:open={confirmOpen} headline={confirming?.kind === 'leave' ? m.org_leave() : m.org_remove()}>
	{#if confirming?.kind === 'leave'}
		<p>{m.org_leave_confirm({ org: org.name })}</p>
		<form id="confirm-form" method="POST" action="?/leave" use:enhance={closeConfirm}></form>
	{:else if confirming?.kind === 'remove'}
		<p>{m.org_remove_confirm({ name: confirming.member.name, org: org.name })}</p>
		<form id="confirm-form" method="POST" action="?/remove" use:enhance={closeConfirm}>
			<input type="hidden" name="user_id" value={confirming.member.user_id} />
		</form>
	{/if}
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (confirmOpen = false)}>{m.org_cancel()}</Button>
		<Button variant="filled" type="submit" form="confirm-form">{confirming?.kind === 'leave' ? m.org_leave() : m.org_remove()}</Button>
	{/snippet}
</Dialog>

<style>
	.org {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.org__inner {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 720px;
		margin: 0 auto;
	}
	.back {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 6px;
		min-height: 44px;
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 600;
		text-decoration: none;
	}
	.back:hover {
		color: var(--md-sys-color-on-surface);
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 16px;
	}
	.head__mark {
		display: grid;
		flex: none;
		place-items: center;
		width: 56px;
		height: 56px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font-size: 1.5rem;
		font-weight: 700;
	}
	.head__text {
		flex: 1;
		min-width: 0;
	}
	.head__title {
		margin: 0;
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface);
	}
	.head__meta {
		margin: 2px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.rename {
		display: flex;
		flex: 1;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.rename :global(.rename__field) {
		flex: 1 1 220px;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.status {
		margin: -8px 0 0;
		min-height: 1.25em;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card {
		padding: 20px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.section-title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.section-lede {
		margin: 4px 0 16px;
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.invite {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 12px;
	}
	.invite :global(.invite__email) {
		width: 100%;
	}
	.roles {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 8px;
		width: 100%;
		margin: 0;
		padding: 0;
		border: none;
	}
	.roles__option {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 10px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		cursor: pointer;
	}
	.roles__option:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.roles__option--on {
		border-color: var(--md-sys-color-primary);
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
	.roles__option input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
	.roles__name {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.roles__hint {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.subhead {
		margin: 24px 0 4px;
		font-size: 0.8125rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
	}
	.rows {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.rows--members {
		margin-top: 8px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.invite-row,
	.member {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 56px;
		padding: 8px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.invite-row:last-child {
		border-bottom: none;
	}
	.invite-row__text,
	.member__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.invite-row__email,
	.member__name {
		overflow: hidden;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.invite-row__meta,
	.member__email {
		overflow: hidden;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.member__you {
		margin-left: 6px;
		padding: 1px 6px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.member__role {
		flex: none;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.leave {
		display: flex;
		justify-content: flex-start;
		padding-top: 8px;
	}
</style>
