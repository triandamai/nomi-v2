<script lang="ts">
	// Asks where projects live: one of the person's organizations, or a new one they name.
	// Posts to the Projects page's chooseOrg / createOrg actions.
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { untrack } from 'svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import type { OrgSummary } from '$lib/types';

	let {
		orgs,
		current = null,
		error = null,
		action = '/projects',
	}: { orgs: OrgSummary[]; current?: string | null; error?: string | null; action?: string } = $props();

	const NEW = 'new';
	// Starts on the organization in use (or the first); the person's pick is theirs after that.
	let choice = $state(untrack(() => current ?? orgs[0]?.id ?? NEW));
	let name = $state('');
	let busy = $state(false);

	const target = $derived(choice === NEW ? `${action}?/createOrg` : `${action}?/chooseOrg`);

	function members(count: number) {
		return count === 1 ? m.org_pick_member() : m.org_pick_members({ count });
	}
</script>

<form
	method="POST"
	action={target}
	class="picker"
	use:enhance={() => {
		busy = true;
		return async ({ update }) => {
			await update();
			busy = false;
		};
	}}
>
	<fieldset class="picker__options">
		<legend class="sr-only">{m.org_pick_title()}</legend>
		{#each orgs as org (org.id)}
			<label class="option" class:option--on={choice === org.id}>
				<input type="radio" name="org_id" value={org.id} bind:group={choice} />
				<span class="option__mark" aria-hidden="true">{org.name.trim().charAt(0).toUpperCase() || '?'}</span>
				<span class="option__text">
					<span class="option__name">{org.name}</span>
					<span class="option__meta">{members(org.members)} · {m.org_pick_projects({ count: org.projects })}</span>
				</span>
			</label>
		{/each}
		<label class="option" class:option--on={choice === NEW}>
			<input type="radio" name="org_id" value={NEW} bind:group={choice} />
			<span class="option__mark option__mark--new" aria-hidden="true"><IconPlus size={18} /></span>
			<span class="option__text">
				<span class="option__name">{m.org_pick_new()}</span>
				<span class="option__meta">{m.org_pick_new_hint()}</span>
			</span>
		</label>
		{#if choice === NEW}
			<div class="picker__new">
				<TextField name="name" label={m.org_name()} bind:value={name} maxlength={80} autocomplete="organization" required />
			</div>
		{/if}
	</fieldset>

	{#if error}
		<p class="picker__error" role="alert">{error}</p>
	{/if}

	<div class="picker__actions">
		<Button type="submit" variant="filled" size="m" disabled={busy || (choice === NEW && !name.trim())}>
			{choice === NEW ? m.org_create() : m.org_pick_continue()}
		</Button>
	</div>
</form>

<style>
	.picker {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.picker__options {
		display: flex;
		flex-direction: column;
		gap: 0;
		margin: 0;
		padding: 0;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		overflow: hidden;
	}
	.option {
		display: flex;
		align-items: center;
		gap: 14px;
		min-height: 64px;
		padding: 10px 16px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.option:first-of-type {
		border-top: none;
	}
	.option:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 5%, transparent);
	}
	.option:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.option--on {
		background: color-mix(in srgb, var(--md-sys-color-primary) 9%, transparent);
	}
	.option input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
	.option__mark {
		display: grid;
		flex: none;
		place-items: center;
		width: 36px;
		height: 36px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
		font-weight: 700;
	}
	.option--on .option__mark {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.option__mark--new {
		background: transparent;
		border: 1px dashed var(--md-sys-color-outline);
	}
	.option__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.option__name {
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.option__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.picker__new {
		padding: 4px 16px 16px 66px;
		background: color-mix(in srgb, var(--md-sys-color-primary) 9%, transparent);
	}
	.picker__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.picker__actions {
		display: flex;
		justify-content: flex-end;
	}
	@media (max-width: 480px) {
		.picker__new {
			padding-left: 16px;
		}
		.picker__actions :global(> *) {
			width: 100%;
		}
	}
</style>
