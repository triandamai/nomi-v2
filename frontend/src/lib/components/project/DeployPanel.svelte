<script lang="ts">
	// Publishing on Shipyard: where the app lives, what runs there, every publish with its
	// build and deploy log, and rolling back to an earlier one.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import IconRocket from '$lib/components/icons/IconRocket.svelte';
	import IconUndo from '$lib/components/icons/IconUndo.svelte';
	import { ago, projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { DeploymentDetail, ProjectPublish, PublishDetail } from '$lib/types';

	let { deploy }: { deploy: ProjectDeploy } = $props();

	const overview = $derived(deploy.overview);
	const locale = getLocale();

	// One publish opened to its deployments, and one deployment to its log.
	let openId = $state<string | null>(null);
	let detail = $state<PublishDetail | null>(null);
	let deployment = $state<DeploymentDetail | null>(null);
	let detailError = $state<string | null>(null);

	async function openPublish(publish: ProjectPublish) {
		if (openId === publish.id) {
			openId = null;
			return;
		}
		openId = publish.id;
		detail = null;
		deployment = null;
		detailError = null;
		try {
			detail = await projectApi<PublishDetail>(deploy.projectId, `deploy/publishes/${publish.id}`);
			// The web service's log is what people want first.
			const first = detail.deployments.find((d) => d.service !== 'db') ?? detail.deployments[0];
			if (first) await openDeployment(publish.environment, first.deployment_id);
		} catch (e) {
			detailError = (e as Error).message;
		}
	}

	async function openDeployment(environment: string, deploymentId: string) {
		try {
			deployment = await projectApi<DeploymentDetail>(deploy.projectId, `deploy/deployments/${deploymentId}?environment=${environment}`);
		} catch (e) {
			detailError = (e as Error).message;
		}
	}

	// The open log follows a running publish.
	$effect(() => {
		const id = openId;
		const running = overview?.publishes.find((p) => p.id === id)?.status === 'running';
		if (!id || !running) return;
		const timer = setInterval(async () => {
			if (document.hidden || !detail) return;
			const fresh = await projectApi<PublishDetail>(deploy.projectId, `deploy/publishes/${id}`).catch(() => null);
			if (fresh) detail = fresh;
			if (deployment) await openDeployment(fresh?.environment ?? 'production', deployment.id);
			else if (fresh?.deployments[0]) await openDeployment(fresh.environment, fresh.deployments[0].deployment_id);
		}, 3000);
		return () => clearInterval(timer);
	});

	let rollbackTarget = $state<ProjectPublish | null>(null);
	let rollbackOpen = $state(false);

	const STATUS: Record<ProjectPublish['status'], () => string> = {
		running: m.pj_status_running,
		succeeded: m.pj_status_succeeded,
		failed: m.pj_status_failed,
	};
	const DATABASE: Record<string, () => string> = {
		postgres: m.pj_db_postgres,
		sqlite: m.pj_db_sqlite,
		none: m.pj_db_none,
	};

	function duration(publish: ProjectPublish): string | null {
		if (!publish.finished_at) return null;
		const seconds = Math.max(0, Math.round((new Date(publish.finished_at).getTime() - new Date(publish.created_at).getTime()) / 1000));
		return seconds < 90 ? `${seconds}s` : `${Math.round(seconds / 60)} min`;
	}

	function logTime(at: string): string {
		return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(new Date(at));
	}

	const liveService = $derived(overview?.live?.services.find((s) => s.type !== 'database' && s.name !== 'db') ?? null);
</script>

{#if !overview}
	<p class="muted">{deploy.error ?? m.pj_loading()}</p>
{:else}
	<section class="summary">
		<div class="summary__main">
			<p class="summary__label">{m.pj_production()}</p>
			{#if overview.url}
				<a class="summary__url" href={overview.url} target="_blank" rel="noopener">{overview.url.replace(/^https?:\/\//, '')}<IconOpenInNew size={16} /></a>
			{:else}
				<p class="summary__url summary__url--none">{m.pj_not_published()}</p>
			{/if}
			<p class="summary__meta">
				{DATABASE[overview.database]()}
				{#if liveService}· {m.pj_replicas({ running: liveService.running, replicas: liveService.replicas })}{/if}
			</p>
		</div>
		{#if overview.available && overview.role === 'admin'}
			<Button variant="filled" size="m" disabled={deploy.busy || deploy.running} onclick={() => deploy.publish()}>
				<IconRocket size={18} />{deploy.running ? m.pj_publishing() : overview.url ? m.pj_publish_again() : m.pj_publish()}
			</Button>
		{/if}
	</section>

	{#if !overview.available}
		<p class="notice">{m.pj_not_available()}</p>
	{:else if overview.role !== 'admin'}
		<p class="notice">{m.pj_admins_only()}</p>
	{/if}
	{#if deploy.error}<p class="error" role="alert">{deploy.error}</p>{/if}

	<h3 class="section-title">{m.pj_history()}</h3>
	{#if overview.publishes.length === 0}
		<p class="muted">{m.pj_history_empty()}</p>
	{:else}
		<ul class="publishes">
			{#each overview.publishes as publish, index (publish.id)}
				{@const open = openId === publish.id}
				<li class="publish" class:publish--open={open}>
					<div class="publish__row">
						<button type="button" class="publish__main" aria-expanded={open} onclick={() => openPublish(publish)}>
							<span class="state state--{publish.status}">{STATUS[publish.status]()}</span>
							<span class="publish__text">
								<span class="publish__title">
									{publish.rollback_of ? m.pj_rollback_title() : m.pj_publish_title({ version: publish.files_version })}
									{#if index === 0 && publish.status === 'succeeded'}<span class="current">{m.pj_current()}</span>{/if}
								</span>
								<span class="publish__meta">
									{m.pj_by({ name: publish.created_by ?? '—', when: ago(publish.created_at, locale) })}
									{#if duration(publish)}· {duration(publish)}{/if}
								</span>
								{#if publish.error}<span class="publish__error">{publish.error}</span>{/if}
							</span>
						</button>
						{#if overview.role === 'admin' && publish.can_roll_back && publish.status === 'succeeded' && index > 0}
							<Button variant="text" size="s" disabled={deploy.busy || deploy.running} onclick={() => { rollbackTarget = publish; rollbackOpen = true; }}>
								<IconUndo size={16} />{m.pj_roll_back()}
							</Button>
						{/if}
					</div>
					{#if open}
						<div class="publish__detail">
							{#if detailError}<p class="error">{detailError}</p>{/if}
							{#if detail && detail.deployments.length > 1}
								<div class="services" role="tablist" aria-label={m.pj_services()}>
									{#each detail.deployments as d (d.deployment_id)}
										<button
											type="button"
											role="tab"
											class="service-tab"
											aria-selected={deployment?.id === d.deployment_id}
											onclick={() => openDeployment(publish.environment, d.deployment_id)}
										>{d.service}</button>
									{/each}
								</div>
							{/if}
							{#if deployment}
								{#if deployment.steps.length > 0}
									<ol class="steps">
										{#each deployment.steps as step (step.name)}
											<li class="step step--{step.status}">{step.name}</li>
										{/each}
									</ol>
								{/if}
								<pre class="log">{#each deployment.logs as line, i (i)}<span class="log__line log__line--{line.level}"><span class="log__time">{logTime(line.at)}</span> {line.message}
</span>{:else}{m.pj_log_empty()}{/each}</pre>
							{:else if !detailError}
								<p class="muted">{publish.status === 'running' ? m.pj_waiting_build() : m.pj_loading()}</p>
							{/if}
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<Dialog bind:open={rollbackOpen} headline={m.pj_roll_back()}>
	{#if rollbackTarget}<p>{m.pj_roll_back_confirm({ when: ago(rollbackTarget.created_at, locale) })}</p>{/if}
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (rollbackOpen = false)}>{m.issues_cancel()}</Button>
		<Button
			variant="filled"
			type="button"
			onclick={async () => {
				rollbackOpen = false;
				if (rollbackTarget) await deploy.rollback(rollbackTarget.id);
			}}>{m.pj_roll_back()}</Button
		>
	{/snippet}
</Dialog>

<style>
	.summary {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
		padding: 16px 18px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.summary__main {
		min-width: 0;
	}
	.summary__label {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.summary__url {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		margin: 2px 0 0;
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.1875rem;
		font-weight: 700;
		overflow-wrap: anywhere;
		text-decoration: none;
	}
	.summary__url:hover {
		text-decoration: underline;
	}
	.summary__url:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.summary__url--none {
		color: var(--md-sys-color-on-surface);
	}
	.summary__meta {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.notice,
	.muted {
		margin: 12px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.error {
		margin: 12px 0 0;
		color: var(--md-sys-color-error);
	}
	.section-title {
		margin: 24px 0 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.publishes {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.publish {
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.publish__row {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.publish__main {
		display: flex;
		flex: 1;
		align-items: flex-start;
		gap: 12px;
		min-width: 0;
		min-height: 56px;
		padding: 10px 8px;
		border: none;
		background: none;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.publish__main:hover,
	.publish--open .publish__main {
		background: var(--md-sys-color-surface-container);
	}
	.publish__main:focus-visible,
	.service-tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.state {
		flex: none;
		min-width: 76px;
		margin-top: 1px;
		padding: 2px 8px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
		text-align: center;
	}
	.state--succeeded {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.state--failed {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.publish__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.publish__title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		font-weight: 600;
	}
	.current {
		padding: 0 6px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.6875rem;
		font-weight: 600;
	}
	.publish__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.publish__error {
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.publish__detail {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 4px 8px 14px;
		background: var(--md-sys-color-surface-container);
	}
	.services {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.service-tab {
		min-height: 32px;
		padding: 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.service-tab[aria-selected='true'] {
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.steps {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 14px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 0.8125rem;
	}
	.step {
		color: var(--md-sys-color-on-surface-variant);
	}
	.step::before {
		content: '○ ';
	}
	.step--success::before,
	.step--succeeded::before {
		content: '✓ ';
		color: var(--md-sys-color-primary);
	}
	.step--failed {
		color: var(--md-sys-color-error);
	}
	.step--failed::before {
		content: '✕ ';
	}
	.step--running::before {
		content: '◐ ';
	}
	.log {
		max-height: 420px;
		margin: 0;
		padding: 12px 14px;
		overflow: auto;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.55;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.log__time {
		color: var(--md-sys-color-on-surface-variant);
	}
	.log__line--error {
		color: var(--md-sys-color-error);
	}
	@media (max-width: 600px) {
		.publish__row {
			flex-wrap: wrap;
		}
		.publish__row :global(.m3-button) {
			margin: 0 0 8px 96px;
		}
	}
</style>
