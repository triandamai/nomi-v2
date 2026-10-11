<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import type { OrgRole } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let name = $state('');

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

		<ul class="list">
			{#each data.orgs as org (org.id)}
				<li>
					<a class="row" href="/organizations/{org.id}">
						<span class="row__mark" aria-hidden="true">{org.name.trim().charAt(0).toUpperCase() || '?'}</span>
						<span class="row__text">
							<span class="row__name">{org.name}</span>
							<span class="row__meta">
								{ROLE[org.role]()} · {members(org.members)} · {m.org_pick_projects({ count: org.projects })}
								{#if org.id === data.projectOrgId}· <strong>{m.orgs_projects_here()}</strong>{/if}
							</span>
						</span>
						<IconChevronRight size={20} />
					</a>
				</li>
			{/each}
		</ul>

		<section class="create" aria-labelledby="create-title">
			<h2 id="create-title" class="create__title">{m.org_new_title()}</h2>
			<p class="create__hint">{m.org_pick_new_hint()}</p>
			<form method="POST" action="?/create" use:enhance class="create__form">
				<TextField name="name" label={m.org_name()} bind:value={name} maxlength={80} required class="create__field" />
				<Button type="submit" variant="filled" size="m" disabled={!name.trim()}>{m.org_create()}</Button>
			</form>
			{#if form?.error}<p class="create__error" role="alert">{form.error}</p>{/if}
		</section>
	</div>
</div>

<style>
	.orgs {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.orgs__inner {
		display: flex;
		flex-direction: column;
		gap: 24px;
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
	.row {
		display: flex;
		align-items: center;
		gap: 14px;
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
	.create__title {
		margin: 0;
		font-size: 1.125rem;
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
