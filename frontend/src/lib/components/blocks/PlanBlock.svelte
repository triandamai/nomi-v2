<!-- frontend/src/lib/components/blocks/PlanBlock.svelte -->
<script lang="ts">
	import { page } from '$app/state';
	import SideSheet from '$lib/components/m3/SideSheet.svelte';
	import { buildAgentPlansFetchUrl } from '$lib/buildAgentPlansFetchUrl';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'plan' }> } = $props();

	type PlanVersion = { id: string; title: string; version: number; content_html: string | null; created_at: string };

	let sheetOpen = $state(false);
	let loading = $state(false);
	let versions = $state<PlanVersion[]>([]);
	let activeVersion = $state<number | null>(null);
	let loadError = $state<string | null>(null);

	async function openSheet() {
		sheetOpen = true;
		if (versions.length > 0) return;
		loading = true;
		loadError = null;
		try {
			const response = await fetch(buildAgentPlansFetchUrl(page.url.pathname, block.agent_session_id));
			if (!response.ok) throw new Error('failed to load');
			const data = (await response.json()) as { plans: PlanVersion[] };
			versions = data.plans;
			activeVersion = block.version;
		} catch {
			loadError = 'Could not load this plan.';
		} finally {
			loading = false;
		}
	}

	const active = $derived(versions.find((v) => v.version === activeVersion) ?? null);
</script>

<button type="button" class="m3-block-card m3-block-card--plan" onclick={openSheet}>
	<span aria-hidden="true">📋</span>
	<span class="md-body-medium">{block.title} · v{block.version}</span>
</button>

<SideSheet bind:open={sheetOpen}>
	{#if loading}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">Loading…</p>
	{:else if loadError}
		<p class="md-body-medium" style="color: var(--md-sys-color-error)">{loadError}</p>
	{:else}
		<div class="m3-plan-versions">
			{#each versions as v (v.id)}
				<button
					type="button"
					class="m3-plan-versions__chip"
					class:m3-plan-versions__chip--active={v.version === activeVersion}
					onclick={() => (activeVersion = v.version)}
				>
					v{v.version}
				</button>
			{/each}
		</div>
		{#if active}
			<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">{active.title}</h2>
			{#if active.content_html !== null}
				<div class="md-body-medium">{@html active.content_html}</div>
			{:else}
				<p class="md-body-medium" style="color: var(--md-sys-color-error)">Could not load this version.</p>
			{/if}
		{/if}
	{/if}
</SideSheet>

<style>
	.m3-block-card--plan {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		cursor: pointer;
		font: inherit;
		color: inherit;
		text-align: left;
	}

	.m3-plan-versions {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 16px;
	}

	.m3-plan-versions__chip {
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		cursor: pointer;
	}

	.m3-plan-versions__chip--active {
		background: var(--md-sys-color-primary);
		border-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
</style>
