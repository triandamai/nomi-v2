<script lang="ts">
	import RoleMenu from '$lib/components/ds/build/RoleMenu.svelte';
	import BuildInvite from '$lib/components/ds/build/BuildInvite.svelte';
	import TextField from '$lib/components/ds/inputs/TextField.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import { page } from '$app/state';
	import { getLocale } from '$lib/paraglide/runtime';
	// Who can open the project: everyone in its organization, or only the people added to it,
	// as members (they build it) or admins (they also publish and manage it).
	import { m } from '$lib/paraglide/messages';
	import ProjectActivity from '$lib/components/project/ProjectActivity.svelte';
	import { projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { ProjectPresence } from '$lib/projectPresence.svelte';
	import type { ProjectMembers, ProjectPerson } from '$lib/types';

	let {
		projectId,
		orgId,
		tab = $bindable('people'),
		presence = null,
		deploy = null,
		projectName = '',
	}: {
		projectId: string;
		orgId: string;
		/** People: who can open the project. Activity: who's here and what's happening. */
		tab?: 'people' | 'activity';
		presence?: ProjectPresence | null;
		deploy?: ProjectDeploy | null;
		/** For the invite's preview. */
		projectName?: string;
	} = $props();

	// Inviting someone to build together (Build together → Roles, Invite email): an address and a
	// role; someone already in the organization is added right away, anyone else gets an invite,
	// shown here as they'll get it.
	let inviteEmail = $state('');
	let inviteRole = $state<'build' | 'prompt' | 'view'>('prompt');
	let inviting = $state(false);
	let inviteError = $state<string | null>(null);
	let sent = $state<{ added: boolean; email: string; role: 'build' | 'prompt' | 'view'; path: string | null; expires_at: string | null } | null>(null);
	const myName = $derived(page.data.profile?.display_name || page.data.profile?.username || (page.data.userEmail ?? '').split('@')[0] || '');
	async function invite(event: SubmitEvent) {
		event.preventDefault();
		if (!inviteEmail.trim()) return;
		inviting = true;
		inviteError = null;
		try {
			sent = await projectApi(projectId, 'invites', { method: 'POST', body: { email: inviteEmail.trim(), role: inviteRole } });
			inviteEmail = '';
			await load();
		} catch (e) {
			inviteError = (e as Error).message;
		} finally {
			inviting = false;
		}
	}

	let data = $state<ProjectMembers | null>(null);
	let error = $state<string | null>(null);
	let busy = $state(false);

	async function load() {
		try {
			data = await projectApi<ProjectMembers>(projectId, 'members');
		} catch (e) {
			error = (e as Error).message;
		}
	}
	$effect(() => {
		void load();
	});

	async function act(run: () => Promise<ProjectMembers>) {
		busy = true;
		error = null;
		try {
			data = await run();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = false;
		}
	}

	const isAdmin = $derived(data?.my_role === 'admin');

	function setAccess(access: 'org' | 'members') {
		void act(() => projectApi(projectId, 'access', { method: 'PUT', body: { access } }));
	}

	// The design system's roles and the project's: build is an admin, prompt a member, view a viewer.
	const DESIGN_ROLE: Record<string, 'build' | 'prompt' | 'view'> = { admin: 'build', member: 'prompt', viewer: 'view' };
	const PROJECT_ROLE = { build: 'admin', prompt: 'member', view: 'viewer' } as const;
	let rolesFor = $state<string | null>(null);
	function roleLabel(person: ProjectPerson): string {
		if (person.is_owner || person.project_role === 'admin') return m.role_build();
		if (person.project_role === 'viewer') return m.role_view();
		if (person.project_role === 'member') return m.role_prompt();
		return canOpen(person) ? m.pj_role_org() : m.pj_role_none();
	}

	function setRole(person: ProjectPerson, role: string) {
		if (role === 'none') void act(() => projectApi(projectId, `members/${person.user_id}`, { method: 'DELETE' }));
		else void act(() => projectApi(projectId, `members/${person.user_id}`, { method: 'PUT', body: { role } }));
	}

	function canOpen(person: ProjectPerson): boolean {
		return data?.access === 'org' || person.is_owner || person.project_role !== null;
	}
</script>

<div class="tabs" role="tablist" aria-label={m.pj_act_tabs()}>
	<button type="button" role="tab" class="tab" aria-selected={tab === 'people'} onclick={() => (tab = 'people')}>{m.pj_act_tab_people()}</button>
	<button type="button" role="tab" class="tab" aria-selected={tab === 'activity'} onclick={() => (tab = 'activity')}>{m.pj_act_tab_activity()}</button>
</div>

{#if tab === 'activity'}
	<ProjectActivity {projectId} {presence} {deploy} />
{:else}

{#if !data}
	<p class="muted">{error ?? m.pj_loading()}</p>
{:else}
	<fieldset class="access" disabled={!isAdmin || busy}>
		<legend class="section-title">{m.pj_access()}</legend>
		<label class="option" class:option--on={data.access === 'org'}>
			<input type="radio" name="access" checked={data.access === 'org'} onchange={() => setAccess('org')} />
			<span><strong>{m.pj_access_org()}</strong><span class="option__hint">{m.pj_access_org_hint()}</span></span>
		</label>
		<label class="option" class:option--on={data.access === 'members'}>
			<input type="radio" name="access" checked={data.access === 'members'} onchange={() => setAccess('members')} />
			<span><strong>{m.pj_access_members()}</strong><span class="option__hint">{m.pj_access_members_hint()}</span></span>
		</label>
	</fieldset>
	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<h3 class="section-title">{m.pj_people()}</h3>
	<ul class="people">
		{#each data.people as person (person.user_id)}
			<li class="person" class:person--out={!canOpen(person)}>
				<span class="person__text">
					<span class="person__name">{person.name}{#if person.user_id === data.me} · {m.pj_you()}{/if}</span>
					<span class="person__meta">{person.email ?? ''}{#if person.is_owner} · {m.pj_started_it()}{/if}</span>
				</span>
				{#if person.is_owner}
					<span class="role">{m.pj_role_admin()}</span>
				{:else if isAdmin}
					<!-- Roles as the design system has them (Build together → Roles): build (publishes),
					     prompt (asks Koda; an admin publishes), view (changes nothing). -->
					<span class="role-pick">
						<button type="button" class="role-button" aria-haspopup="true" aria-expanded={rolesFor === person.user_id} aria-label={m.pj_role_for({ name: person.name })} disabled={busy} onclick={() => (rolesFor = rolesFor === person.user_id ? null : person.user_id)}>
							{roleLabel(person)}
						</button>
						{#if rolesFor === person.user_id}
							<span class="role-menu">
								<RoleMenu
									value={DESIGN_ROLE[person.project_role ?? 'member'] ?? 'prompt'}
									onchange={(role) => {
										rolesFor = null;
										setRole(person, PROJECT_ROLE[role]);
									}}
								/>
								{#if person.project_role}
									<button type="button" class="role-remove" onclick={() => { rolesFor = null; setRole(person, 'none'); }}>{data.access === 'org' ? m.pj_role_org() : m.pj_role_none()}</button>
								{/if}
							</span>
						{/if}
					</span>
				{:else}
					<span class="role">{roleLabel(person)}</span>
				{/if}
			</li>
		{/each}
	</ul>
	{#if isAdmin}
		<form class="invite-form" onsubmit={invite}>
			<h3 class="invite-form__title">{m.pj_invite_title()}</h3>
			<TextField label={m.pj_invite_email()} type="email" bind:value={inviteEmail} />
			<RoleMenu value={inviteRole} onchange={(role) => (inviteRole = role)} />
			<span><Button label={m.pj_invite_send()} type="submit" disabled={inviting || !inviteEmail.trim()} /></span>
			{#if inviteError}<p class="error" role="alert">{inviteError}</p>{/if}
		</form>
		{#if sent?.added}
			<p class="muted small" role="status">{m.pj_invite_added({ email: sent.email })}</p>
		{:else if sent?.path}
			<p class="muted small" role="status">{m.pj_invite_sent_preview()}</p>
			<BuildInvite
				from={myName}
				project={projectName}
				role={sent.role}
				href={sent.path}
				sentTo={sent.email}
				expires={sent.expires_at ? new Date(sent.expires_at).toLocaleDateString(getLocale(), { dateStyle: 'medium' }) : ''}
			/>
		{/if}
	{/if}
	<p class="muted small">{m.pj_invite_hint()} <a href="/organizations/{orgId}" target="_blank" rel="noopener">{m.pj_open_org()}</a></p>
{/if}
{/if}

<style>
	.tabs {
		display: flex;
		gap: 4px;
		margin-bottom: 16px;
		padding: 3px;
		width: fit-content;
		border-radius: 999px;
		background: var(--md-sys-color-surface-container-low);
	}
	.tab {
		min-height: 36px;
		padding: 6px 16px;
		border: 0;
		border-radius: 999px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: 500 0.875rem var(--md-ref-typeface-plain);
		cursor: pointer;
	}
	.tab:hover {
		color: var(--md-sys-color-on-surface);
	}
	.tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.tab[aria-selected='true'] {
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.section-title {
		margin: 18px 0 8px;
		padding: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.access {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: none;
	}
	.access .section-title {
		margin-top: 0;
	}
	.option {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		cursor: pointer;
	}
	.option--on {
		border-color: var(--md-sys-color-primary);
		background: color-mix(in srgb, var(--md-sys-color-primary) 6%, transparent);
	}
	.option input {
		margin-top: 3px;
		accent-color: var(--md-sys-color-primary);
	}
	.option > span {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.option__hint {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.access:disabled .option {
		cursor: default;
	}
	.people {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.person {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 56px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.person--out .person__text {
		opacity: 0.6;
	}
	.person__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}
	.person__name {
		font-weight: 600;
	}
	.person__meta {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.role {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.muted {
		margin: 12px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.small {
		font-size: 0.8125rem;
	}
	.muted a {
		color: var(--md-sys-color-primary);
	}
	.error {
		margin: 8px 0 0;
		color: var(--md-sys-color-error);
	}
	.role-pick {
		position: relative;
	}
	.role-button {
		min-height: 32px;
		padding: 0 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: inherit;
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}
	.role-button:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.role-menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		z-index: 10;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 4px;
	}
	.role-remove {
		padding: 4px 8px;
		border: none;
		background: var(--md-sys-color-surface-container);
		border-radius: var(--md-sys-shape-corner-small);
		color: var(--md-sys-color-error);
		font: inherit;
		font-size: 0.8125rem;
		cursor: pointer;
	}
	.invite-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: 20px;
	}
	.invite-form__title {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 600;
	}
</style>
