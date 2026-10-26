<script lang="ts">
	// One publish, opened from the Deployments list: for a failed one, where it broke first, then
	// its steps by phase with each step's part of the log. Follows a running publish live and
	// lands on the step that failed.
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconArrowBack from '$lib/components/icons/IconArrowBack.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import IconRocket from '$lib/components/icons/IconRocket.svelte';
	import IconUndo from '$lib/components/icons/IconUndo.svelte';
	import StatusSquare from '$lib/components/project/StatusSquare.svelte';
	import { environmentName, phaseTitle } from '$lib/components/project/deployLabels';
	import { durationLabel, focusStep, groupSteps, normalizeDeployment, stepLog } from '$lib/deployView';
	import { ago, projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { DeploymentDetail, ProjectPublish, PublishDetail } from '$lib/types';

	/** How often an open, running deployment is fetched again (its log isn't in the events). */
	const FOLLOW_MS = 3000;

	let {
		deploy,
		publish,
		liveVersion,
		canPublish,
		canRollBack,
		onback,
		onaskfix,
		onrollback,
	}: {
		deploy: ProjectDeploy;
		publish: ProjectPublish;
		liveVersion: number | null;
		canPublish: boolean;
		canRollBack: boolean;
		onback: () => void;
		onaskfix?: (text: string) => void;
		onrollback: () => void;
	} = $props();

	const locale = getLocale();
	let detail = $state<PublishDetail | null>(null);
	let deploymentId = $state<string | null>(null);
	let deployment = $state<DeploymentDetail | null>(null);
	let error = $state<string | null>(null);
	let selectedStep = $state<string | null>(null);
	let wholeLog = $state(false);

	const failure = $derived(publish.failure ?? null);
	const status = $derived(
		publish.status === 'running' ? m.pj_dep_tag_running() : publish.status === 'failed' ? m.pj_dep_tag_failed() : m.pj_status_succeeded(),
	);
	// The web service first: its log is what people want.
	const deployments = $derived([...(detail?.deployments ?? [])].sort((a, b) => Number(a.service === 'db') - Number(b.service === 'db')));
	const groups = $derived(groupSteps(deployment?.steps ?? []));
	const step = $derived(deployment?.steps.find((s) => s.name === selectedStep) ?? null);
	const shownLog = $derived(wholeLog || !step ? (deployment?.logs ?? []) : stepLog(deployment?.logs ?? [], step));

	async function loadDetail() {
		try {
			const fresh = await projectApi<PublishDetail>(deploy.projectId, `deploy/publishes/${publish.id}`);
			detail = fresh;
			const failedService = failure?.deployment_id;
			const first = fresh.deployments.find((d) => d.deployment_id === failedService) ?? fresh.deployments.find((d) => d.service !== 'db') ?? fresh.deployments[0];
			const current = untrack(() => deploymentId);
			const id = current && fresh.deployments.some((d) => d.deployment_id === current) ? current : (first?.deployment_id ?? null);
			if (id) await loadDeployment(id);
			error = null;
		} catch (e) {
			error = (e as Error).message;
		}
	}

	async function loadDeployment(id: string) {
		const body = await projectApi<unknown>(deploy.projectId, `deploy/deployments/${id}?environment=${encodeURIComponent(publish.environment)}`);
		const next = normalizeDeployment(body, id);
		const changedDeployment = untrack(() => deploymentId) !== id;
		deploymentId = id;
		deployment = next;
		// Land on the step that matters, and move there when a running one fails.
		const current = untrack(() => selectedStep);
		const focus = focusStep(next.steps);
		const currentStatus = next.steps.find((s) => s.name === current)?.status;
		if (changedDeployment || !current || (focus?.status === 'failed' && currentStatus !== 'failed')) selectedStep = focus?.name ?? null;
	}

	function pickDeployment(id: string) {
		selectedStep = null;
		wholeLog = false;
		void loadDeployment(id).catch((e) => (error = (e as Error).message));
	}

	$effect(() => {
		void publish.id;
		untrack(() => void loadDetail());
	});
	// A change pushed for the project (a step finished, the publish ended): fetch it again.
	$effect(() => {
		if (deploy.changes) untrack(() => void loadDetail());
	});
	$effect(() => {
		if (publish.status !== 'running') return;
		const timer = setInterval(() => {
			if (!document.hidden && deploymentId) void loadDeployment(deploymentId).catch(() => null);
			else if (!document.hidden) void loadDetail();
		}, FOLLOW_MS);
		return () => clearInterval(timer);
	});

	function logTime(at: string): string {
		const date = new Date(at);
		return isNaN(date.getTime()) ? '' : new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(date);
	}

	function askFix() {
		onaskfix?.(
			m.pj_dep_fix_request({
				version: publish.files_version,
				env: environmentName(publish.environment),
				phase: failure ? phaseTitle(failure.phase) : '—',
				step: failure?.step ?? '—',
				message: failure?.message || publish.error || '—',
			}),
		);
	}
</script>

<div class="head">
	<IconButton onclick={onback} aria-label={m.pj_dep_back()} title={m.pj_dep_back()}><IconArrowBack size={20} /></IconButton>
	<div class="head__text">
		<h3 class="head__title">{m.pj_publish_title({ version: publish.files_version })}</h3>
		<p class="head__meta">{m.pj_dep_status_line({ status, when: ago(publish.created_at, locale), env: environmentName(publish.environment) })}</p>
	</div>
</div>

{#if publish.status === 'failed'}
	<div class="callout" role="alert">
		<p class="callout__title">
			{#if failure}{m.pj_dep_broke_title({ phase: phaseTitle(failure.phase), step: failure.step })}{:else}{m.pj_dep_tag_failed()}{/if}
		</p>
		{#if failure?.message || publish.error}<p class="callout__message">{failure?.message || publish.error}</p>{/if}
		<p class="callout__meta">
			{#if liveVersion !== null}{m.pj_dep_still_live({ version: liveVersion })}{/if}
			{m.pj_dep_koda_told()}
		</p>
	</div>
	<div class="actions">
		{#if onaskfix}<Button variant="filled" size="m" onclick={askFix}>{m.pj_dep_ask_fix()}</Button>{/if}
		{#if canPublish}
			<Button variant="text" size="m" disabled={deploy.busy || deploy.running} onclick={() => deploy.publish(publish.environment)}>
				<IconRocket size={18} />{m.pj_dep_publish_again()}
			</Button>
		{/if}
	</div>
{:else if publish.status === 'succeeded'}
	<div class="actions">
		{#if publish.url}
			<a class="went-live" href={publish.url} target="_blank" rel="noopener">{m.pj_dep_live_at({ url: publish.url.replace(/^https?:\/\//, '') })}<IconOpenInNew size={14} /></a>
		{/if}
		{#if canRollBack}
			<Button variant="tonal" size="m" disabled={deploy.busy || deploy.running} onclick={onrollback}><IconUndo size={18} />{m.pj_dep_roll_back_here()}</Button>
		{/if}
	</div>
{/if}

{#if error}<p class="error" role="alert">{error}</p>{/if}

{#if deployments.length > 1}
	<div class="tabs" role="tablist" aria-label={m.pj_services()}>
		{#each deployments as d (d.deployment_id)}
			<button type="button" role="tab" class="tab" aria-selected={d.deployment_id === deploymentId} onclick={() => pickDeployment(d.deployment_id)}>
				<StatusSquare status={d.status} size={18} />{d.service}
			</button>
		{/each}
	</div>
{/if}

{#if deployment}
	{#if groups.length > 0}
		<section class="steps" aria-label={m.pj_dep_steps()}>
			{#each groups as group (group.phase)}
				<p class="phase">{phaseTitle(group.phase)}</p>
				<ul>
					{#each group.steps as s (s.name)}
						{@const time = s.started_at ? durationLabel(s.started_at, s.finished_at) : null}
						<li>
							<button
								type="button"
								class="step"
								aria-current={s.name === selectedStep && !wholeLog ? 'step' : undefined}
								onclick={() => {
									selectedStep = s.name;
									wholeLog = false;
								}}
							>
								<StatusSquare status={s.status} size={24} label={s.status} />
								<span class="step__name">{s.name}</span>
								<span class="step__time">{s.status === 'skipped' ? m.pj_dep_skipped() : (time ?? '')}</span>
							</button>
						</li>
					{/each}
				</ul>
			{/each}
		</section>
	{/if}

	<div class="log-head">
		<p class="log-head__title">{wholeLog || !step ? m.pj_dep_whole_log() : step.name}</p>
		{#if step}
			<Button variant="text" size="s" onclick={() => (wholeLog = !wholeLog)}>{wholeLog ? m.pj_dep_step_only() : m.pj_dep_whole_log()}</Button>
		{/if}
	</div>
	{#if step && !wholeLog && step.error}<p class="step-error">{step.error}</p>{/if}
	{#if shownLog === null || shownLog.length === 0}
		<p class="muted">{step && !wholeLog ? m.pj_dep_no_step_log() : publish.status === 'running' ? m.pj_waiting_build() : m.pj_log_empty()}</p>
	{:else}
		<pre class="log">{#each shownLog as line, i (i)}<span class="log__line" class:log__line--error={line.level === 'error'} class:log__line--warn={line.level === 'warn' || line.level === 'warning'}><span class="log__time">{logTime(line.at)}</span> {line.message}
</span>{/each}</pre>
	{/if}
{:else if !error}
	<p class="muted">{publish.status === 'running' ? m.pj_waiting_build() : (failure?.tail?.length ? '' : m.pj_log_empty())}</p>
	{#if failure?.tail?.length}<pre class="log">{failure.tail.join('\n')}</pre>{/if}
{/if}

{#if failure?.runtime_logs?.length}
	<p class="log-head__title runtime">{m.pj_failure_runtime()}</p>
	<pre class="log">{failure.runtime_logs.join('\n')}</pre>
{/if}

<style>
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: -4px 0 14px -8px;
	}
	.head__text {
		min-width: 0;
	}
	.head__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.head__meta {
		margin: 2px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.callout {
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: color-mix(in srgb, var(--md-sys-color-error-container) 75%, var(--md-sys-color-surface-container-lowest));
		color: var(--md-sys-color-on-error-container);
	}
	.callout p {
		margin: 0;
		line-height: 1.45;
	}
	.callout__title {
		font-weight: 700;
	}
	.callout__message {
		margin-top: 2px !important;
		font-size: 0.875rem;
		overflow-wrap: anywhere;
	}
	.callout__meta {
		margin-top: 8px !important;
		font-size: 0.8125rem;
		opacity: 0.85;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		margin: 12px 0 16px;
	}
	.went-live {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--md-sys-color-primary);
		font-size: 0.875rem;
		text-decoration: none;
		overflow-wrap: anywhere;
	}
	.went-live:hover {
		text-decoration: underline;
	}
	.went-live:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.error {
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.muted {
		margin: 8px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-bottom: 12px;
	}
	.tab {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
		padding: 6px 12px 6px 8px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 10px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: 500 0.8125rem var(--md-ref-typeface-plain);
		cursor: pointer;
	}
	.tab:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.tab[aria-selected='true'] {
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.steps {
		padding: 6px 0;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.steps ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.phase {
		margin: 0;
		padding: 10px 16px 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.step {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 12px;
		align-items: center;
		width: 100%;
		min-height: 44px;
		padding: 8px 16px;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: 0.875rem var(--md-ref-typeface-plain);
		text-align: left;
		cursor: pointer;
	}
	.step:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.step:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.step[aria-current='step'] {
		background: var(--md-sys-color-surface-container-low);
		font-weight: 600;
	}
	.step__name {
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.step__time {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.log-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin: 16px 0 8px;
	}
	.log-head__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.runtime {
		margin: 16px 0 8px;
	}
	.step-error {
		margin: 0 0 8px;
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.log {
		max-height: 420px;
		margin: 0;
		padding: 14px 16px;
		overflow: auto;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		font: 0.78rem/1.55 ui-monospace, 'JetBrains Mono', Menlo, monospace;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.log__time {
		opacity: 0.6;
	}
	.log__line--error {
		color: var(--md-sys-color-error-container);
	}
	.log__line--warn {
		color: var(--md-sys-color-tertiary-container);
	}
</style>
