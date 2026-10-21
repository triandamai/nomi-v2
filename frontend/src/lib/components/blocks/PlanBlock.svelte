<!-- frontend/src/lib/components/blocks/PlanBlock.svelte -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import { onDestroy, onMount, untrack } from 'svelte';
	import SideSheet from '$lib/components/m3/SideSheet.svelte';
	import { buildAgentPlansFetchUrl } from '$lib/buildAgentPlansFetchUrl';
	import type { ChecklistItem } from '$lib/planChecklist';
	import { planRefresh } from '$lib/planRefresh.svelte';
	import { planStatusLine } from '$lib/planStatus';
	import type { BuildCheck, ContentBlock, PlanBuild, PlanState } from '$lib/types';
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
		/** Missing from servers that don't report it. */
		state?: PlanState;
	};
	type PlanVersions = { plans: PlanVersion[]; build?: PlanBuild | null };

	let sheetOpen = $state(false);
	let loading = $state(false);
	let versions = $state<PlanVersion[]>([]);
	let build = $state<PlanBuild | null>(null);
	let activeVersion = $state<number | null>(null);
	let loadError = $state<string | null>(null);

	function apply(data: PlanVersions) {
		versions = data.plans;
		build = data.build ?? null;
	}

	async function loadVersions() {
		if (versions.length > 0 || loading) return;
		loading = true;
		loadError = null;
		try {
			const response = await fetch(buildAgentPlansFetchUrl(page.url.pathname, block.agent_session_id));
			if (!response.ok) throw new Error('failed to load');
			apply((await response.json()) as PlanVersions);
			activeVersion = block.version;
		} catch {
			loadError = m.err_load_plan();
		} finally {
			loading = false;
		}
	}

	// Something in the chat may have moved this plan on (a build started or ended, a new version
	// came): look again, quietly, keeping what's shown until the answer is in.
	let refreshTimer: ReturnType<typeof setTimeout> | null = null;
	let seenRevision = untrack(() => planRefresh.revision);
	$effect(() => {
		const revision = planRefresh.revision;
		if (revision === seenRevision) return;
		seenRevision = revision;
		if (refreshTimer) clearTimeout(refreshTimer);
		refreshTimer = setTimeout(reload, 300);
	});
	onDestroy(() => {
		if (refreshTimer) clearTimeout(refreshTimer);
	});

	async function reload() {
		refreshTimer = null;
		if (versions.length === 0) return loadVersions();
		try {
			const response = await fetch(buildAgentPlansFetchUrl(page.url.pathname, block.agent_session_id));
			if (response.ok) apply((await response.json()) as PlanVersions);
		} catch {
			// Best-effort: the bubble keeps what it showed, and the next change tries again.
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

	const latestVersion = $derived(versions.reduce((latest, v) => Math.max(latest, v.version), 0));
	// The build only ever follows the newest version.
	const status = $derived(shownVersion ? planStatusLine(shownVersion.state, shownVersion.version === latestVersion ? build : null, latestVersion) : null);
	const sheetBuild = $derived(active && active.version === latestVersion ? build : null);

	function checkOutcome(check: BuildCheck): string {
		switch (check.outcome) {
			case 'passed':
				return m.plan_build_passed();
			case 'failed':
				return m.plan_build_failed({ code: check.exit_code === null ? '?' : String(check.exit_code) });
			case 'timed_out':
				return m.plan_build_timed_out();
			case 'stale':
				return m.plan_build_stale();
			default:
				return m.plan_build_not_run();
		}
	}

	// Ticking a step saves it into this version's plan; the bubble updates straight away and
	// takes it back if the save fails.
	let saveError = $state<string | null>(null);
	async function toggleItem(index: number, done: boolean) {
		const plan = shownVersion;
		if (!plan) return;
		const before = versions;
		versions = versions.map((v) => (v.id === plan.id ? { ...v, checklist: withItem(v.checklist, index, done) } : v));
		saveError = null;
		try {
			const response = await fetch(buildAgentPlansFetchUrl(page.url.pathname, block.agent_session_id), {
				method: 'PATCH',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ planId: plan.id, index, done }),
			});
			if (!response.ok) throw new Error('failed to save');
			apply((await response.json()) as PlanVersions);
		} catch {
			versions = before;
			saveError = m.err_save_plan_item();
		}
	}

	/** The checklist with one item ticked or unticked; the first open step stays the current one. */
	function withItem(items: ChecklistItem[], index: number, done: boolean): ChecklistItem[] {
		let currentFound = false;
		return items.map((item, i) => {
			const isDone = i === index ? done : item.status === 'done';
			if (isDone) return { ...item, status: 'done' };
			const status = currentFound ? 'pending' : 'in_progress';
			currentFound = true;
			return { ...item, status };
		});
	}
</script>

<ChecklistBubble
	{agent}
	heading={m.plan_draft({ agent: agent ?? 'Nomi' })}
	items={shownVersion?.checklist ?? []}
	maxItems={6}
	ontoggle={toggleItem}
	working={status ? status.tone === 'working' : undefined}
>
	<p class="m3-plan-title">{block.title}</p>
	{#if status}
		<p class="m3-plan-status" data-tone={status.tone} role="status">
			<span class="m3-plan-status__label">{status.label}</span>{status.detail ? ` · ${status.detail}` : ''}
		</p>
	{/if}
	{#if shownVersion && shownVersion.checklist.length === 0 && shownVersion.excerpt}
		<p class="m3-plan-excerpt">{shownVersion.excerpt}</p>
	{/if}
	{#snippet footer()}
		{#if saveError}<p class="m3-plan-error" role="alert">{saveError}</p>{/if}
		<button type="button" class="m3-plan-open" onclick={openSheet}>
			{m.plan_open()}
			<span class="nomi-meta">v{block.version}</span>
		</button>
	{/snippet}
</ChecklistBubble>

<SideSheet bind:open={sheetOpen} title={m.plan_draft({ agent: agent ?? 'Nomi' })}>
	{#if loading}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">{m.common_loading()}</p>
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
				<p class="md-body-medium" style="color: var(--md-sys-color-error)">{m.plan_version_failed()}</p>
			{/if}
			{#if sheetBuild?.report}
				<section class="m3-plan-build" aria-label={m.plan_build_heading()}>
					<h3 class="md-title-small">{m.plan_build_heading()}</h3>
					<ul class="m3-plan-build__checks">
						{#each sheetBuild.report.checks as check (check.command)}
							<li data-outcome={check.outcome}>{m.plan_build_check({ command: check.command, outcome: checkOutcome(check) })}</li>
						{/each}
					</ul>
					{#if sheetBuild.report.changed_files.length > 0}
						<details class="m3-plan-build__files">
							<summary>{m.plan_build_changed({ count: String(sheetBuild.report.changed_files.length) })}</summary>
							<ul>
								{#each sheetBuild.report.changed_files as path (path)}<li><code>{path}</code></li>{/each}
							</ul>
						</details>
					{/if}
				</section>
			{/if}
		{/if}
	{/if}
</SideSheet>

<style>
	.m3-plan-title {
		margin: 0;
		font-weight: 650;
		font-size: 0.9375rem;
	}
	.m3-plan-excerpt {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-plan-status {
		margin: 0;
		font-size: 0.8125rem;
		line-height: 1.4;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-plan-status__label {
		font-weight: 650;
		color: var(--md-sys-color-on-surface);
	}
	.m3-plan-status[data-tone='done'] .m3-plan-status__label {
		color: var(--md-sys-color-primary);
	}
	.m3-plan-status[data-tone='error'],
	.m3-plan-status[data-tone='error'] .m3-plan-status__label {
		color: var(--md-sys-color-error);
	}
	.m3-plan-status[data-tone='muted'] .m3-plan-status__label {
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-plan-build {
		margin-top: 20px;
		padding-top: 16px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-plan-build h3 {
		margin: 0 0 8px;
		color: var(--md-sys-color-on-surface);
	}
	.m3-plan-build__checks {
		margin: 0;
		padding-left: 18px;
		font-size: 0.875rem;
		line-height: 1.6;
		color: var(--md-sys-color-on-surface);
	}
	.m3-plan-build__checks li[data-outcome='failed'] {
		color: var(--md-sys-color-error);
	}
	.m3-plan-build__files {
		margin-top: 10px;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-plan-build__files summary {
		cursor: pointer;
		min-height: 44px;
		display: flex;
		align-items: center;
	}
	.m3-plan-build__files summary:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.m3-plan-build__files ul {
		margin: 0;
		padding-left: 18px;
	}
	.m3-plan-error {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
	.m3-plan-open {
		align-self: flex-start;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 40px;
		padding: 0 14px;
		border: none;
		border-radius: 20px;
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
		margin-bottom: 14px;
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
