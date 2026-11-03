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
	import PlanCard from '$lib/components/ds/chat/PlanCard.svelte';
	import DraftPlanCard from '$lib/components/ds/chat/DraftPlanCard.svelte';
	import PlanApproval from '$lib/components/ds/chat/PlanApproval.svelte';
	import QuickReplies from '$lib/components/ds/chat/QuickReplies.svelte';
	import TextField from '$lib/components/ds/inputs/TextField.svelte';
	import { useChatSend } from '$lib/chatSend';
	import { crewId } from '$lib/crewId';
	import type { CrewRosterMember } from '$lib/crew';
	import type { StepState } from '$lib/components/ds/chat/stepState';
	import BuildPlan from '$lib/components/ds/build/BuildPlan.svelte';
	import BuildRunning from '$lib/components/ds/build/BuildRunning.svelte';
	import BuildDone from '$lib/components/ds/build/BuildDone.svelte';
	import BuildShareDialog from '$lib/components/ds/build/BuildShareDialog.svelte';
	import { projectApi } from '$lib/projectApi';
	import { getLocale } from '$lib/paraglide/runtime';

	let { block, agent = null, answered = false }: { block: Extract<ContentBlock, { kind: 'plan' }>; agent?: string | null; answered?: boolean } = $props();

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
	const sheetStatus = $derived(active ? planStatusLine(active.state, sheetBuild, latestVersion) : null);
	// How far the version's steps have come, when it has steps to tick.
	const progress = $derived.by(() => {
		const total = active?.checklist.length ?? 0;
		if (!active || total === 0) return null;
		return { done: active.checklist.filter((item) => item.status === 'done').length, total };
	});

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

	// The design system's plan cards (Chat responses → Plan, Draft plan, Plan approval). A plan
	// still being proposed is a draft the person shapes: every change goes to the crew as a
	// message, and the planning agent writes the next version. Once it runs, it's the running plan
	// with each step's real state.
	const send = useChatSend();
	const owner = $derived({ agent: crewId(agent), name: agent ?? 'Nomi' });
	const items = $derived(shownVersion?.checklist ?? []);
	const STATE: Record<ChecklistItem['status'], StepState> = { done: 'done', in_progress: 'running', pending: 'waiting' };
	// Only a project's plan is built by the crew; anywhere else a plan is the person's own
	// checklist, ticked off as they go (the checklist bubble below).
	const inProject = $derived(page.url.pathname.startsWith('/projects/'));
	const draft = $derived(inProject && shownVersion?.state === 'proposed' && shownVersion.version === latestVersion && !answered && !!send);
	const building = $derived(inProject && !!shownVersion && shownVersion.state !== 'proposed' && shownVersion.version === latestVersion && !!build);
	const running = $derived(building && shownVersion?.state === 'active');
	const roster = $derived(((page.data.crew ?? []) as CrewRosterMember[]).filter((member) => member.agent_type !== 'personality'));

	// What the person is doing to the draft right now: picking who does a step, adding a step,
	// asking for changes, or choosing how it runs.
	let editing = $state<{ kind: 'assign'; index: number } | { kind: 'add' } | { kind: 'changes' } | { kind: 'run' } | null>(null);
	let note = $state('');

	function ask(text: string) {
		if (!send) return;
		send(text);
		editing = null;
		note = '';
	}
	const label = (index: number) => items[index]?.text ?? '';

	// A project's build, as the Build sheet draws it: the estimate before anything runs
	// (BuildPlan; its steps can be edited in the draft card), Koda building (BuildRunning, the real
	// step and its time), and the live link when it's done (BuildDone, shared with BuildShare).
	const project = $derived(page.data.project as { id: string; name: string; description?: string | null } | null | undefined);
	let editingSteps = $state(false);
	const done = $derived(building && shownVersion?.state === 'completed');
	const currentIndex = $derived(Math.max(0, items.findIndex((item) => item.status === 'in_progress')));
	const doneCount = $derived(items.filter((item) => item.status === 'done').length);
	let now = $state(Date.now());
	$effect(() => {
		if (!running) return;
		const timer = setInterval(() => (now = Date.now()), 1000);
		return () => clearInterval(timer);
	});
	const clockOf = (iso: string | undefined) => (iso ? new Date(iso).toLocaleTimeString(getLocale(), { hour: '2-digit', minute: '2-digit' }) : '');
	function duration(from: string | undefined, to: number): string {
		const start = from ? Date.parse(from) : NaN;
		if (Number.isNaN(start)) return '';
		const seconds = Math.max(0, Math.floor((to - start) / 1000));
		return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
	}

	// The live link and how it's shared: the project's showcase setting is who can find it (public
	// lists it in the showcase, unlisted is the link only) and its blurb is the preview's text.
	type Showcase = { showcase: boolean; blurb: string | null; url: string | null; can_edit: boolean };
	let showcase = $state<Showcase | null>(null);
	$effect(() => {
		if (!done || !project) return;
		void projectApi<Showcase>(project.id, 'showcase').then((found) => (showcase = found)).catch(() => (showcase = null));
	});
	const liveUrl = $derived(showcase?.url ?? (project ? `${page.url.origin}/projects/${project.id}/app` : ''));
	let shareOpen = $state(false);
	let shareError = $state<string | null>(null);
	async function setVisibility(visibility: 'public' | 'unlisted') {
		if (!project || !showcase) return;
		shareError = null;
		try {
			showcase = await projectApi<Showcase>(project.id, 'showcase', { method: 'PUT', body: { showcase: visibility === 'public', blurb: showcase.blurb } });
		} catch (e) {
			shareError = (e as Error).message;
		}
	}
	let editingBlurb = $state(false);
	let blurb = $state('');
	async function saveBlurb() {
		if (!project || !showcase) return;
		try {
			showcase = await projectApi<Showcase>(project.id, 'showcase', { method: 'PUT', body: { showcase: showcase.showcase, blurb } });
			editingBlurb = false;
		} catch (e) {
			shareError = (e as Error).message;
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

{#if shownVersion && items.length > 0 && draft && !editingSteps && editing === null}
	<BuildPlan
		project={project?.name ?? block.title}
		summary={block.title}
		steps={items.map((item) => item.text)}
		outcome={shownVersion.excerpt || (project?.description ?? '')}
		options={[{ title: m.plan_run_all(), detail: m.plan_run_all_detail(), recommended: true }]}
		cost={m.build_cost_note({ count: String(items.length) })}
		onrun={(choice) => ask('keepPlanning' in choice ? choice.keepPlanning : m.plan_ask_run_all())}
	/>
	<button type="button" class="plan-edit__link" onclick={() => (editingSteps = true)}>{m.plan_edit_steps()}</button>
{:else if shownVersion && items.length > 0 && draft}
	<DraftPlanCard
		intro={block.title}
		steps={items.map((item) => ({ label: item.text, owner }))}
		estimate={m.plan_draft_meta({ count: String(items.length), version: String(block.version) })}
		onmove={(from, to) => ask(m.plan_ask_move({ from: String(from + 1), step: label(from), to: String(to + 1) }))}
		onremove={(index) => ask(m.plan_ask_remove({ index: String(index + 1), step: label(index) }))}
		onassign={(index) => (editing = { kind: 'assign', index })}
		onadd={() => (editing = { kind: 'add' })}
		onchanges={() => (editing = { kind: 'changes' })}
		onrun={() => (editing = { kind: 'run' })}
		ondiscard={() => ask(m.plan_ask_discard())}
	/>
	{#if editing?.kind === 'assign'}
		{@const index = editing.index}
		<div class="plan-edit">
			<p class="plan-edit__label">{m.plan_assign_label({ index: String(index + 1) })}</p>
			<QuickReplies replies={roster.map((member) => member.name)} onpick={(name) => ask(m.plan_ask_assign({ index: String(index + 1), step: label(index), name }))} />
		</div>
	{:else if editing?.kind === 'add' || editing?.kind === 'changes'}
		{@const adding = editing.kind === 'add'}
		<form
			class="plan-edit"
			onsubmit={(event) => {
				event.preventDefault();
				if (note.trim()) ask(adding ? m.plan_ask_add({ step: note.trim() }) : note.trim());
			}}
		>
			<TextField label={adding ? m.plan_add_label() : m.plan_changes_label()} bind:value={note} />
		</form>
	{:else if editing?.kind === 'run'}
		<PlanApproval
			who={owner.name}
			options={[{ title: m.plan_run_all(), detail: m.plan_run_all_detail(), recommended: true }]}
			onrun={(choice) => ask('keepPlanning' in choice ? choice.keepPlanning : m.plan_ask_run_all())}
		/>
	{/if}
{:else if shownVersion && items.length > 0 && running && build}
	<BuildRunning
		started={clockOf(build.started_at ?? build.updated_at)}
		step={items[currentIndex]?.text ?? block.title}
		elapsed={duration(build.started_at, now)}
		current={Math.min(items.length, doneCount + 1)}
		total={items.length}
		label={block.title}
		onstop={send ? () => send('stop') : undefined}
	/>
{:else if shownVersion && done && build}
	<BuildDone
		meta={[duration(build.started_at, build.finished_at ? Date.parse(build.finished_at) : Date.parse(build.updated_at)), build.report?.changed_files.length ? m.files_changed_count({ count: String(build.report.changed_files.length) }) : ''].filter(Boolean).join(' · ')}
		url={liveUrl}
		summary={block.title}
		ondetails={openSheet}
		onshare={() => (shareOpen = true)}
	/>
	{#if showcase}
		<BuildShareDialog
			bind:open={shareOpen}
			url={liveUrl}
			preview={{ title: project?.name ?? block.title, description: showcase.blurb ?? project?.description ?? undefined }}
			visibility={showcase.showcase ? 'public' : 'unlisted'}
			onvisibility={showcase.can_edit && showcase.url ? setVisibility : undefined}
			oneditpreview={showcase.can_edit ? () => { blurb = showcase?.blurb ?? ''; editingBlurb = true; shareOpen = false; } : undefined}
			oninvite={() => { shareOpen = false; window.dispatchEvent(new CustomEvent('nomi:open-sheet', { detail: 'members' })); }}
		/>
	{/if}
	{#if editingBlurb}
		<form class="plan-edit" onsubmit={(event) => { event.preventDefault(); void saveBlurb(); }}>
			<TextField label={m.bshare_edit_preview()} bind:value={blurb} />
		</form>
	{/if}
	{#if shareError}<p class="m3-plan-error" role="alert">{shareError}</p>{/if}
{:else if shownVersion && items.length > 0 && building}
	<PlanCard
		subject={block.title}
		steps={items.map((item) => ({ label: item.text, state: STATE[item.status], owner }))}
		onstop={running && send ? () => send('stop') : undefined}
	/>
{:else}
	<ChecklistBubble {agent} heading={m.plan_draft({ agent: agent ?? 'Nomi' })} {items} maxItems={6} ontoggle={toggleItem} working={status ? status.tone === 'working' : undefined}>
		<p class="m3-plan-title">{block.title}</p>
		{#if shownVersion && items.length === 0 && shownVersion.excerpt}<p class="m3-plan-excerpt">{shownVersion.excerpt}</p>{/if}
	</ChecklistBubble>
{/if}
<div class="plan-foot">
	{#if status}
		<p class="m3-plan-status" data-tone={status.tone} role="status">
			<span class="m3-plan-status__label">{status.label}</span>{status.detail ? ` · ${status.detail}` : ''}
		</p>
	{/if}
	{#if saveError}<p class="m3-plan-error" role="alert">{saveError}</p>{/if}
	<button type="button" class="m3-plan-open" onclick={openSheet}>
		{m.plan_open()}
		<span class="nomi-meta">v{block.version}</span>
	</button>
</div>

<SideSheet bind:open={sheetOpen} wide title={m.plan_draft({ agent: agent ?? 'Nomi' })}>
	{#if loading}
		<p class="plan-sheet__quiet">{m.common_loading()}</p>
	{:else if loadError}
		<p class="plan-sheet__error" role="alert">{loadError}</p>
	{:else if active}
		<article class="plan-sheet">
			<header class="plan-sheet__head">
				<h2 class="plan-sheet__title">{active.title}</h2>
				<div class="plan-sheet__meta">
					<span>{m.plan_sheet_meta({ agent: agent ?? 'Nomi', version: String(active.version) })}</span>
					{#if versions.length > 1}
						<label class="plan-sheet__version">
							<span class="sr-only">{m.plan_sheet_version()}</span>
							<select bind:value={activeVersion}>
								{#each [...versions].sort((a, b) => b.version - a.version) as v (v.id)}
									<option value={v.version}>
										{v.version === latestVersion ? m.plan_sheet_version_latest({ version: String(v.version) }) : m.plan_sheet_version_option({ version: String(v.version) })}
									</option>
								{/each}
							</select>
						</label>
					{/if}
				</div>
				{#if sheetStatus}
					<p class="plan-sheet__status" data-tone={sheetStatus.tone} role="status">
						<strong>{sheetStatus.label}</strong>{sheetStatus.detail ? `: ${sheetStatus.detail}` : ''}
					</p>
				{/if}
				{#if progress}
					<div class="plan-sheet__progress">
						<span>{m.plan_sheet_progress({ done: String(progress.done), total: String(progress.total) })}</span>
						<span class="plan-sheet__bar" aria-hidden="true"><span style="width: {(progress.done / progress.total) * 100}%"></span></span>
					</div>
				{/if}
			</header>

			{#if active.content_html !== null}
				<div class="plan-sheet__doc message-markdown">{@html active.content_html}</div>
			{:else}
				<p class="plan-sheet__error">{m.plan_version_failed()}</p>
			{/if}

			{#if sheetBuild?.report}
				<section class="plan-sheet__build" aria-label={m.plan_sheet_build()}>
					<h3>{m.plan_sheet_build()}</h3>
					<ul class="plan-sheet__checks">
						{#each sheetBuild.report.checks as check (check.command)}
							<li data-outcome={check.outcome}>
								<code>{check.command}</code>
								<span class="plan-sheet__outcome">{checkOutcome(check)}</span>
							</li>
						{/each}
					</ul>
					{#if sheetBuild.report.changed_files.length > 0}
						<details class="plan-sheet__files">
							<summary>{m.plan_sheet_files_changed({ count: String(sheetBuild.report.changed_files.length) })}</summary>
							<ul>
								{#each sheetBuild.report.changed_files as path (path)}<li><code>{path}</code></li>{/each}
							</ul>
						</details>
					{/if}
				</section>
			{/if}
		</article>
	{/if}
</SideSheet>

<style>
	.plan-edit {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 8px;
	}
	.plan-edit__label {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.plan-edit__link {
		margin-top: 6px;
		padding: 0;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		text-decoration: underline;
		cursor: pointer;
	}
	.plan-foot {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		margin-top: 8px;
	}
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

	/* The plan as a document to read: its title leads, one plain line on where it stands, then the
	   plan at a reading width. */
	.plan-sheet {
		max-width: 68ch;
		margin: 4px auto 0;
		padding-bottom: 24px;
	}
	.plan-sheet__head {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding-bottom: 20px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.plan-sheet__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.5rem, 1.2rem + 1.2vw, 2rem);
		font-weight: 700;
		line-height: 1.15;
		letter-spacing: -0.015em;
		text-wrap: balance;
	}
	.plan-sheet__meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.plan-sheet__version select {
		min-height: 36px;
		padding: 0 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		cursor: pointer;
	}
	.plan-sheet__version select:hover {
		border-color: var(--md-sys-color-outline);
	}
	.plan-sheet__version select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	@media (max-width: 600px) {
		.plan-sheet__version select {
			min-height: 44px;
		}
	}
	.plan-sheet__status {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.9375rem;
		line-height: 1.5;
	}
	.plan-sheet__status strong {
		color: var(--md-sys-color-on-surface);
		font-weight: 650;
	}
	.plan-sheet__status[data-tone='done'] strong {
		color: var(--md-sys-color-primary);
	}
	.plan-sheet__status[data-tone='error'],
	.plan-sheet__status[data-tone='error'] strong {
		color: var(--md-sys-color-error);
	}
	.plan-sheet__progress {
		display: flex;
		flex-direction: column;
		gap: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.plan-sheet__bar {
		display: block;
		height: 4px;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
	}
	.plan-sheet__bar span {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: var(--md-sys-color-primary);
		transition: width var(--nomi-motion-spatial-fast);
	}

	.plan-sheet__doc {
		padding-top: 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		line-height: 1.65;
	}
	.plan-sheet__doc :global(h1),
	.plan-sheet__doc :global(h2),
	.plan-sheet__doc :global(h3) {
		margin: 1.6em 0 0.4em;
		font-family: var(--md-ref-typeface-brand);
		line-height: 1.25;
		letter-spacing: -0.01em;
	}
	.plan-sheet__doc :global(h1) {
		font-size: 1.375rem;
	}
	.plan-sheet__doc :global(h2) {
		font-size: 1.1875rem;
	}
	.plan-sheet__doc :global(h3) {
		font-size: 1rem;
	}
	.plan-sheet__doc :global(p) {
		margin: 0.6em 0;
	}
	.plan-sheet__doc :global(ul),
	.plan-sheet__doc :global(ol) {
		margin: 0.5em 0;
		padding-left: 1.3em;
	}
	.plan-sheet__doc :global(li) {
		margin: 0.25em 0;
	}
	.plan-sheet__doc :global(li:has(> input[type='checkbox'])) {
		margin-left: -1.3em;
		list-style: none;
	}
	.plan-sheet__doc :global(input[type='checkbox']) {
		margin: 0 0.5em 0 0;
		accent-color: var(--md-sys-color-primary);
		vertical-align: -2px;
	}
	.plan-sheet__doc :global(code) {
		padding: 1px 5px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-surface-container-highest);
		font-size: 0.875em;
	}
	.plan-sheet__doc :global(strong) {
		font-weight: 650;
	}
	.plan-sheet__doc :global(a) {
		color: var(--md-sys-color-primary);
	}

	.plan-sheet__build {
		margin-top: 28px;
		padding-top: 18px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.plan-sheet__build h3 {
		margin: 0 0 6px;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1rem;
	}
	.plan-sheet__checks {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.plan-sheet__checks li {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 16px;
		padding: 10px 0;
		font-size: 0.875rem;
	}
	.plan-sheet__checks li + li {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.plan-sheet__checks code {
		min-width: 0;
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface);
	}
	.plan-sheet__outcome {
		flex: none;
		color: var(--md-sys-color-on-surface-variant);
	}
	.plan-sheet__checks li[data-outcome='passed'] .plan-sheet__outcome {
		color: var(--md-sys-color-primary);
	}
	.plan-sheet__checks li[data-outcome='failed'] .plan-sheet__outcome {
		color: var(--md-sys-color-error);
	}
	.plan-sheet__files {
		margin-top: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.plan-sheet__files summary {
		display: flex;
		align-items: center;
		min-height: 44px;
		cursor: pointer;
	}
	.plan-sheet__files summary:hover {
		color: var(--md-sys-color-on-surface);
	}
	.plan-sheet__files summary:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.plan-sheet__files ul {
		margin: 0;
		padding-left: 18px;
		line-height: 1.7;
	}
	.plan-sheet__quiet {
		color: var(--md-sys-color-on-surface-variant);
	}
	.plan-sheet__error {
		color: var(--md-sys-color-error);
	}
</style>
