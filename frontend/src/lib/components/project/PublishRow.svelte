<script lang="ts">
	// One publish in the Deployments list: its state, its version, who and when, and for a failed
	// one where it broke and why, right in the row.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconUndo from '$lib/components/icons/IconUndo.svelte';
	import StatusSquare from '$lib/components/project/StatusSquare.svelte';
	import { phaseTitle } from '$lib/components/project/deployLabels';
	import { durationLabel } from '$lib/deployView';
	import { ago } from '$lib/projectApi';
	import type { ProjectPublish } from '$lib/types';

	let {
		publish,
		canRollBack,
		busy = false,
		onopen,
		onrollback,
	}: { publish: ProjectPublish; canRollBack: boolean; busy?: boolean; onopen: () => void; onrollback: () => void } = $props();

	const locale = getLocale();
	const who = $derived(publish.started_by_agent ?? publish.created_by ?? '—');
	const failure = $derived(publish.failure ?? null);
	const reason = $derived(failure?.message || publish.error || null);
	const tag = $derived(
		publish.status === 'running'
			? m.pj_dep_tag_running()
			: publish.status === 'failed'
				? m.pj_dep_tag_failed()
				: publish.is_live
					? m.pj_dep_tag_live()
					: null,
	);
	const detail = $derived(
		publish.status === 'failed' && failure
			? m.pj_dep_broke_at({ phase: phaseTitle(failure.phase), step: failure.step })
			: durationLabel(publish.created_at, publish.finished_at),
	);
</script>

<li class="row" class:row--failed={publish.status === 'failed'}>
	<button type="button" class="row__main" onclick={onopen}>
		<StatusSquare status={publish.status} />
		<span class="row__text">
			<span class="row__title">
				{m.pj_publish_title({ version: publish.files_version })}
				{#if tag}<span class="tag" class:tag--live={publish.is_live && publish.status === 'succeeded'}>{tag}</span>{/if}
				{#if publish.rollback_of_version !== null}<span class="tag">{m.pj_dep_tag_rollback({ version: publish.rollback_of_version })}</span>{/if}
			</span>
			<span class="row__meta">
				{m.pj_dep_meta({ when: ago(publish.created_at, locale), name: who })}{#if detail}&nbsp;· {detail}{/if}
			</span>
			{#if publish.status === 'failed' && reason}<span class="row__error">{reason}</span>{/if}
		</span>
		<span class="row__chevron"><IconChevronRight size={18} /></span>
	</button>
	{#if canRollBack}
		<div class="row__action">
			<Button variant="text" size="s" disabled={busy} onclick={onrollback}><IconUndo size={16} />{m.pj_roll_back()}</Button>
		</div>
	{/if}
</li>

<style>
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		border-top: 1px solid var(--md-sys-color-surface-container-low);
	}
	.row:first-child {
		border-top: 0;
	}
	.row__main {
		display: grid;
		flex: 1;
		grid-template-columns: auto 1fr auto;
		gap: 14px;
		align-items: center;
		min-width: 0;
		min-height: 64px;
		padding: 12px 16px;
		border: 0;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.row__main:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.row__main:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.row__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
	}
	.tag {
		padding: 1px 8px;
		border-radius: 6px;
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.tag--live {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.row__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.row__error {
		margin-top: 6px;
		padding: 8px 10px;
		border-radius: 10px;
		background: color-mix(in srgb, var(--md-sys-color-error-container) 70%, transparent);
		color: var(--md-sys-color-on-error-container);
		font-size: 0.8125rem;
		line-height: 1.4;
		overflow-wrap: anywhere;
	}
	.row__chevron {
		display: flex;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row__action {
		padding: 0 12px 0 0;
	}
	@media (max-width: 600px) {
		.row__action {
			width: 100%;
			padding: 0 12px 10px 62px;
		}
	}
</style>
