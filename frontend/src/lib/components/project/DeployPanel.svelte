<script lang="ts">
	// Publishing on Shipyard: what's live in each environment, every publish with where a failed
	// one broke, and going back to an earlier version that worked. Tapping a publish opens its
	// details (steps and logs) in place of the list.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconOpenInNew from '$lib/components/icons/IconOpenInNew.svelte';
	import IconRocket from '$lib/components/icons/IconRocket.svelte';
	import PublishDetails from '$lib/components/project/PublishDetails.svelte';
	import PublishRow from '$lib/components/project/PublishRow.svelte';
	import { environmentName } from '$lib/components/project/deployLabels';
	import { canRollBack, environmentsOf } from '$lib/deployView';
	import { ago } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { ProjectPublish } from '$lib/types';

	let { deploy, onaskfix }: { deploy: ProjectDeploy; onaskfix?: (text: string) => void } = $props();

	const locale = getLocale();
	const overview = $derived(deploy.overview);
	const role = $derived(overview?.role ?? 'member');
	const environments = $derived(environmentsOf(overview?.publishes ?? []));
	let chosenEnvironment = $state<string | null>(null);
	const environment = $derived(chosenEnvironment && environments.includes(chosenEnvironment) ? chosenEnvironment : (environments[0] ?? 'production'));
	const publishes = $derived((overview?.publishes ?? []).filter((p) => p.environment === environment));
	const live = $derived(publishes.find((p) => p.is_live) ?? null);
	const canPublish = $derived(Boolean(overview?.available) && role === 'admin');

	let openId = $state<string | null>(null);
	const opened = $derived(openId ? (overview?.publishes.find((p) => p.id === openId) ?? null) : null);

	let rollbackTarget = $state<ProjectPublish | null>(null);
	let rollbackOpen = $state(false);
	function askRollback(publish: ProjectPublish) {
		rollbackTarget = publish;
		rollbackOpen = true;
	}
</script>

{#if !overview}
	<p class="muted">{deploy.error ?? m.pj_loading()}</p>
{:else if opened}
	<PublishDetails
		{deploy}
		publish={opened}
		liveVersion={opened.environment === environment ? (live?.files_version ?? null) : null}
		{canPublish}
		canRollBack={canRollBack(opened, role)}
		onback={() => (openId = null)}
		{onaskfix}
		onrollback={() => askRollback(opened)}
	/>
{:else}
	{#if !overview.available}
		<p class="notice">{m.pj_not_available()}</p>
	{:else if role !== 'admin'}
		<p class="notice">{m.pj_admins_only()}</p>
	{/if}
	{#if deploy.error}<p class="error" role="alert">{deploy.error}</p>{/if}

	{#if environments.length > 1}
		<div class="chips" role="tablist" aria-label={m.pj_dep_environments()}>
			{#each environments as env (env)}
				<button type="button" role="tab" class="chip" aria-selected={env === environment} onclick={() => (chosenEnvironment = env)}>
					{environmentName(env)}
				</button>
			{/each}
		</div>
	{/if}

	<section class="live">
		{#if live}
			<div class="live__text">
				<h3 class="live__title">{m.pj_dep_live_since({ when: ago(live.finished_at ?? live.created_at, locale) })}</h3>
				{#if live.url}
					<a class="live__url" href={live.url} target="_blank" rel="noopener">{live.url.replace(/^https?:\/\//, '')}<IconOpenInNew size={14} /></a>
				{/if}
				<p class="live__meta">{m.pj_dep_version_by({ version: live.files_version, name: live.started_by_agent ?? live.created_by ?? '—' })}</p>
			</div>
			{#if live.url}<Button variant="tonal" size="m" href={live.url} target="_blank" rel="noopener">{m.pj_dep_open()}</Button>{/if}
		{:else}
			<p class="live__empty">{m.pj_dep_nothing_live({ env: environmentName(environment) })}</p>
			{#if canPublish}
				<Button variant="filled" size="m" disabled={deploy.busy || deploy.running} onclick={() => deploy.publish()}>
					<IconRocket size={18} />{deploy.running ? m.pj_publishing() : m.pj_publish()}
				</Button>
			{/if}
		{/if}
	</section>

	{#if publishes.length === 0}
		<p class="muted">{m.pj_history_empty()}</p>
	{:else}
		<ul class="list" aria-label={m.pj_history()}>
			{#each publishes as publish (publish.id)}
				<PublishRow
					{publish}
					canRollBack={canRollBack(publish, role)}
					busy={deploy.busy || deploy.running}
					onopen={() => (openId = publish.id)}
					onrollback={() => askRollback(publish)}
				/>
			{/each}
		</ul>
	{/if}
{/if}

<Dialog bind:open={rollbackOpen} headline={m.pj_roll_back()}>
	{#if rollbackTarget}<p>{m.pj_dep_roll_back_confirm({ version: rollbackTarget.files_version })}</p>{/if}
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (rollbackOpen = false)}>{m.issues_cancel()}</Button>
		<Button
			variant="filled"
			type="button"
			onclick={async () => {
				rollbackOpen = false;
				if (rollbackTarget) await deploy.rollback(rollbackTarget.id);
				openId = null;
			}}>{m.pj_roll_back()}</Button
		>
	{/snippet}
</Dialog>

<style>
	.muted {
		margin: 8px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.notice {
		margin: 0 0 12px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-bottom: 14px;
	}
	.chip {
		min-height: 32px;
		padding: 6px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: 500 0.8125rem var(--md-ref-typeface-plain);
		cursor: pointer;
	}
	.chip:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.chip:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.chip[aria-selected='true'] {
		border-color: transparent;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.live {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 12px 16px;
		margin-bottom: 16px;
		padding: 16px 18px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.live__text {
		min-width: 0;
	}
	.live__title {
		margin: 0 0 2px;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-weight: 600;
	}
	.live__url {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--md-sys-color-primary);
		font-size: 0.875rem;
		text-decoration: none;
		overflow-wrap: anywhere;
	}
	.live__url:hover {
		text-decoration: underline;
	}
	.live__url:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.live__meta,
	.live__empty {
		margin: 2px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.live__empty {
		font-size: 0.9375rem;
	}
	.list {
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		list-style: none;
	}
</style>
