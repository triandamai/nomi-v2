<!-- frontend/src/lib/components/blocks/PlanBlock.svelte -->
<script lang="ts">
	import { page } from '$app/state';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
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
	<AgentShape agent="planning" size={36} />
	<span class="m3-plan-text">
		<span class="m3-plan-title">{block.title}</span>
		<span class="nomi-meta">Plan · v{block.version} · open</span>
	</span>
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
		gap: 14px;
		width: 100%;
		max-width: 480px;
		padding: 14px 18px 14px 14px;
		border: none;
		border-radius: var(--nomi-shape-bubble-start);
		background: color-mix(in srgb, #7ab0ff 16%, var(--md-sys-color-surface-container-lowest));
		cursor: pointer;
		font: inherit;
		color: var(--md-sys-color-on-surface);
		text-align: left;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.m3-block-card--plan:hover {
		border-radius: var(--md-sys-shape-corner-large);
	}
	.m3-plan-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.m3-plan-title {
		font-weight: 650;
		font-size: 1rem;
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
