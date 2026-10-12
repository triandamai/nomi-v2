<script lang="ts">
	// The project's dangerous actions: taking the published app offline, and deleting the
	// whole project everywhere.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';

	let { deploy, projectName }: { deploy: ProjectDeploy; projectName: string } = $props();

	const isAdmin = $derived(deploy.isAdmin);
	const live = $derived(Boolean(deploy.overview?.live));

	let offlineOpen = $state(false);
	let deleteOpen = $state(false);
	let typed = $state('');
	let deleting = $state(false);
	let error = $state<string | null>(null);

	async function remove() {
		deleting = true;
		error = null;
		try {
			await projectApi(deploy.projectId, '', { method: 'DELETE' });
			location.href = '/projects';
		} catch (e) {
			error = (e as Error).message;
			deleting = false;
		}
	}
</script>

{#if !isAdmin}
	<p class="muted">{m.pj_admins_only()}</p>
{:else}
	<section class="danger">
		<div class="danger__row">
			<div>
				<h3 class="danger__title">{m.pj_offline()}</h3>
				<p class="muted">{m.pj_offline_hint()}</p>
			</div>
			<Button variant="outlined" size="s" disabled={!live || deploy.busy} onclick={() => (offlineOpen = true)}>{m.pj_offline()}</Button>
		</div>
		<div class="danger__row">
			<div>
				<h3 class="danger__title">{m.pj_delete()}</h3>
				<p class="muted">{m.pj_delete_hint()}</p>
			</div>
			<Button variant="filled" size="s" class="danger-button" onclick={() => { typed = ''; deleteOpen = true; }}>{m.pj_delete()}</Button>
		</div>
	</section>
	{#if error || deploy.error}<p class="error" role="alert">{error ?? deploy.error}</p>{/if}
{/if}

<Dialog bind:open={offlineOpen} headline={m.pj_offline()}>
	<p>{m.pj_offline_confirm()}</p>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (offlineOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="button" onclick={async () => { offlineOpen = false; await deploy.takeOffline(); }}>{m.pj_offline()}</Button>
	{/snippet}
</Dialog>

<Dialog bind:open={deleteOpen} headline={m.pj_delete()}>
	<p>{m.pj_delete_confirm({ name: projectName })}</p>
	<TextField name="confirm" label={m.pj_delete_type()} bind:value={typed} />
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (deleteOpen = false)}>{m.issues_cancel()}</Button>
		<Button variant="filled" type="button" disabled={typed.trim() !== projectName || deleting} onclick={remove}>{m.pj_delete()}</Button>
	{/snippet}
</Dialog>

<style>
	.danger {
		border: 1px solid color-mix(in srgb, var(--md-sys-color-error) 40%, var(--md-sys-color-outline-variant));
		border-radius: var(--md-sys-shape-corner-large);
	}
	.danger__row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 16px 18px;
	}
	.danger__row + .danger__row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.danger__title {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.muted {
		margin: 4px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		margin: 12px 0 0;
		color: var(--md-sys-color-error);
	}
	.danger :global(.danger-button) {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
</style>
