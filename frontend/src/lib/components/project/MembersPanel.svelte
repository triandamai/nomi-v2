<script lang="ts">
	// Who can open the project: everyone in its organization, or only the people added to it,
	// as members (they build it) or admins (they also publish and manage it).
	import { m } from '$lib/paraglide/messages';
	import { projectApi } from '$lib/projectApi';
	import type { ProjectMembers, ProjectPerson } from '$lib/types';

	let { projectId, orgId }: { projectId: string; orgId: string } = $props();

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

	function setRole(person: ProjectPerson, role: string) {
		if (role === 'none') void act(() => projectApi(projectId, `members/${person.user_id}`, { method: 'DELETE' }));
		else void act(() => projectApi(projectId, `members/${person.user_id}`, { method: 'PUT', body: { role } }));
	}

	function canOpen(person: ProjectPerson): boolean {
		return data?.access === 'org' || person.is_owner || person.project_role !== null;
	}
</script>

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
					<select
						class="role-select"
						aria-label={m.pj_role_for({ name: person.name })}
						value={person.project_role ?? 'none'}
						disabled={busy}
						onchange={(e) => setRole(person, (e.currentTarget as HTMLSelectElement).value)}
					>
						<option value="none">{data.access === 'org' ? m.pj_role_org() : m.pj_role_none()}</option>
						<option value="member">{m.pj_role_member()}</option>
						<option value="admin">{m.pj_role_admin()}</option>
					</select>
				{:else}
					<span class="role">{person.project_role === 'admin' ? m.pj_role_admin() : canOpen(person) ? m.pj_role_member() : m.pj_role_none()}</span>
				{/if}
			</li>
		{/each}
	</ul>
	<p class="muted small">{m.pj_invite_hint()} <a href="/organizations/{orgId}" target="_blank" rel="noopener">{m.pj_open_org()}</a></p>
{/if}

<style>
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
	.role-select {
		min-height: 40px;
		padding: 0 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
	}
	.role-select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
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
</style>
