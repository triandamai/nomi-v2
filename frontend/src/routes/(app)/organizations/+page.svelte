<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import { activityLine } from '$lib/orgs';
	import type { OrgRole } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let name = $state('');
	// "" is the workspace last used.
	const initialOpen = () => data.openOrgId ?? '';
	let openIn = $state(initialOpen());
	const openOptions = $derived([
		{ value: '', label: m.orgs_open_last_used() },
		...data.orgs.map((org) => ({ value: org.id, label: org.name })),
	]);

	const ROLE: Record<OrgRole, () => string> = {
		owner: m.org_role_owner,
		admin: m.org_role_admin,
		member: m.org_role_member,
	};

	function members(count: number) {
		return count === 1 ? m.org_pick_member() : m.org_pick_members({ count });
	}
</script>

<div class="orgs">
	<div class="orgs__inner">
		<header>
			<h1 class="md-display-small orgs__title">{m.orgs_title()}</h1>
			<p class="md-body-large orgs__lede">{m.orgs_lede()}</p>
		</header>

		{#if data.invites.length > 0}
			<section class="invites" aria-labelledby="invites-title">
				<h2 id="invites-title" class="create__title">{m.orgs_invites_title()}</h2>
				<ul class="list">
					{#each data.invites as invite (invite.code)}
						<li class="item invite">
							<span class="row__mark" aria-hidden="true">{invite.org_name.trim().charAt(0).toUpperCase() || '?'}</span>
							<span class="row__text">
								<span class="row__name">{m.orgs_invite_from({ inviter: invite.inviter, org: invite.org_name })}</span>
								<span class="row__meta">
									{ROLE[invite.role]()} · {m.org_pick_projects({ count: invite.projects })}
									{#if invite.project}· {m.orgs_invite_project({ project: invite.project.name })}{/if}
								</span>
							</span>
							<form method="POST" action="?/decline" use:enhance class="switch">
								<input type="hidden" name="code" value={invite.code} />
								<Button type="submit" variant="text" size="s">{m.orgs_invite_decline()}</Button>
							</form>
							<form method="POST" action="?/join" use:enhance class="switch">
								<input type="hidden" name="code" value={invite.code} />
								<Button type="submit" variant="filled" size="s">{m.orgs_invite_join()}</Button>
							</form>
						</li>
					{/each}
				</ul>
				{#if form && 'inviteError' in form && form.inviteError}<p class="create__error" role="alert">{form.inviteError}</p>{/if}
			</section>
		{/if}

		<ul class="list">
			{#each data.orgs as org (org.id)}
				<li class="item">
					<a class="row" href="/organizations/{org.id}">
						<span class="row__mark" aria-hidden="true">{org.name.trim().charAt(0).toUpperCase() || '?'}</span>
						<span class="row__text">
							<span class="row__name">
								{org.name}
								{#if org.is_personal}<span class="tag">{m.org_personal()}</span>{/if}
							</span>
							<span class="row__meta">
								{ROLE[org.role]()} · {members(org.members)} · {m.org_pick_projects({ count: org.projects })}
								{#if org.id === data.projectOrgId}· <strong>{m.orgs_projects_here()}</strong>{/if}
							</span>
							{#if activityLine(org.last_activity)}<span class="row__meta">{activityLine(org.last_activity)}</span>{/if}
						</span>
						{#if (org.member_avatars ?? []).length > 1}
							<span class="faces" aria-hidden="true">
								{#each org.member_avatars as face, i (i)}
									<Avatar name={face.name} avatarUrl={face.avatar_url} size={24} />
								{/each}
							</span>
						{/if}
						<IconChevronRight size={20} />
					</a>
					{#if org.id === data.activeOrgId}
						<span class="active">{m.org_active()}</span>
					{:else}
						<form method="POST" action="?/switch" use:enhance class="switch">
							<input type="hidden" name="org_id" value={org.id} />
							<Button type="submit" variant="text" size="s">{m.org_switch()}</Button>
						</form>
					{/if}
				</li>
			{/each}
		</ul>
		{#if form && 'switchError' in form && form.switchError}<p class="create__error" role="alert">{form.switchError}</p>{/if}

		{#if data.orgs.length > 1}
			<section class="create" aria-labelledby="open-title">
				<h2 id="open-title" class="create__title">{m.orgs_open_in()}</h2>
				<p class="create__hint">{m.orgs_open_in_hint()}</p>
				<form method="POST" action="?/openIn" use:enhance={() => async ({ update }) => update({ reset: false })} class="create__form">
					<Select name="org_id" label={m.orgs_open_in()} options={openOptions} bind:value={openIn} class="create__field" />
					<Button type="submit" variant="tonal" size="m" disabled={openIn === (data.openOrgId ?? '')}>{m.org_save()}</Button>
				</form>
				<p class="create__hint" role="status" aria-live="polite">{form && 'openSaved' in form && form.openSaved ? m.orgs_open_saved() : ''}</p>
				{#if form && 'openError' in form && form.openError}<p class="create__error" role="alert">{form.openError}</p>{/if}
			</section>
		{/if}

		<section class="create" aria-labelledby="create-title">
			<h2 id="create-title" class="create__title">{m.org_new_title()}</h2>
			<p class="create__hint">{m.org_pick_new_hint()}</p>
			<form method="POST" action="?/create" use:enhance class="create__form">
				<TextField name="name" label={m.org_name()} bind:value={name} maxlength={80} required class="create__field" />
				<Button type="submit" variant="filled" size="m" disabled={!name.trim()}>{m.org_create()}</Button>
			</form>
			{#if form && 'error' in form && form.error}<p class="create__error" role="alert">{form.error}</p>{/if}
		</section>
	</div>
</div>

<style>
	.orgs {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.orgs__inner {
		display: flex;
		flex-direction: column;
		gap: 20px;
		max-width: 720px;
		margin: 0 auto;
	}
	.orgs__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.orgs__lede {
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.list {
		margin: 0;
		padding: 0;
		list-style: none;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.list li {
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.item {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.item .row {
		flex: 1;
		min-width: 0;
	}
	.switch {
		flex: none;
	}
	.active {
		flex: none;
		padding: 0 12px;
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 64px;
		padding: 10px 8px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.row:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.row__mark {
		display: grid;
		flex: none;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-weight: 700;
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__name {
		font-weight: 700;
	}
	.row__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__meta strong {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.tag {
		margin-left: 6px;
		padding: 1px 6px;
		border-radius: var(--md-sys-shape-corner-extra-small, 4px);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.625rem;
		font-weight: 500;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		vertical-align: middle;
	}
	.faces {
		display: flex;
		flex: none;
	}
	.faces :global(.m3-avatar + .m3-avatar) {
		margin-left: -6px;
	}
	.invite {
		padding: 10px 8px;
		gap: 12px;
	}
	.invites {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.create__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.create__hint {
		margin: 4px 0 12px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.create__form {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 12px;
	}
	.create__form :global(.create__field) {
		flex: 1 1 240px;
	}
	.create__error {
		margin: 8px 0 0;
		color: var(--md-sys-color-error);
	}
</style>
