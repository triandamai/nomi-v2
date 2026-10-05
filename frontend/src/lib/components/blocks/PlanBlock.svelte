<!-- frontend/src/lib/components/blocks/PlanBlock.svelte -->
<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import SideSheet from '$lib/components/m3/SideSheet.svelte';
	import { buildAgentPlansFetchUrl } from '$lib/buildAgentPlansFetchUrl';
	import type { ChecklistItem } from '$lib/planChecklist';
	import type { ContentBlock } from '$lib/types';
	import ChecklistBubble from './ChecklistBubble.svelte';

	let { block, agent = null }: { block: Extract<ContentBlock, { kind: 'plan' }>; agent?: string | null } = $props();

	type PlanVersion = {
		id: string;
		title: string;
		version: number;
		content_html: string | null;
		checklist: ChecklistItem[];
		excerpt: string;
		created_at: string;
	};

	let sheetOpen = $state(false);
	let loading = $state(false);
	let versions = $state<PlanVersion[]>([]);
	let activeVersion = $state<number | null>(null);
	let loadError = $state<string | null>(null);

	async function loadVersions() {
		if (versions.length > 0 || loading) return;
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

	function openSheet() {
		sheetOpen = true;
		loadVersions();
	}

	// The bubble shows this version's checklist (or opening lines), so load it straight away.
	onMount(loadVersions);

	const shownVersion = $derived(versions.find((v) => v.version === block.version) ?? null);

	const active = $derived(versions.find((v) => v.version === activeVersion) ?? null);
</script>

<ChecklistBubble {agent} heading="{agent ?? 'Nomi'}’s draft" items={shownVersion?.checklist ?? []} maxItems={6}>
	<p class="m3-plan-title">{block.title}</p>
	{#if shownVersion && shownVersion.checklist.length === 0 && shownVersion.excerpt}
		<p class="m3-plan-excerpt">{shownVersion.excerpt}</p>
	{/if}
	{#snippet footer()}
		<button type="button" class="m3-plan-open" onclick={openSheet}>
			Open plan
			<span class="nomi-meta">v{block.version}</span>
		</button>
	{/snippet}
</ChecklistBubble>

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
	.m3-plan-title {
		margin: 0;
		font-weight: 650;
		font-size: 1rem;
	}
	.m3-plan-excerpt {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-plan-open {
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 40px;
		padding: 0 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 650;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.m3-plan-open:hover {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.m3-plan-open:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
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
