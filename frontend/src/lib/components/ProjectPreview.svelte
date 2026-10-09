<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import type { ProjectRuntime, RuntimePhase } from '$lib/projectRuntime.svelte';

	// The project's live preview, from its dev server running in the browser (WebContainer),
	// with what it's doing while it gets there. Static projects, and browsers WebContainer can't
	// run in, get the plain file preview instead.
	let { projectId, runtime }: { projectId: string; runtime: ProjectRuntime | null } = $props();

	let frame = $state<HTMLIFrameElement | null>(null);

	const PHASE_LABEL: Record<RuntimePhase, () => string> = {
		unsupported: m.project_phase_unsupported,
		booting: m.project_phase_booting,
		loading: m.project_phase_loading,
		waiting: m.project_phase_waiting,
		installing: m.project_phase_installing,
		checking: m.project_phase_checking,
		starting: m.project_phase_starting,
		running: m.project_phase_running,
		failed: m.project_phase_failed,
		stopped: m.project_phase_stopped,
	};

	const lastLines = $derived(runtime ? runtime.log.trimEnd().split('\n').slice(-6).join('\n') : '');

	function reload() {
		if (frame && runtime?.previewUrl) frame.src = runtime.previewUrl;
	}
</script>

{#if !runtime || runtime.phase === 'unsupported'}
	<div class="preview">
		{#if runtime}<p class="preview__note">{m.project_unsupported_body()}</p>{/if}
		<iframe title={m.project_preview_title()} src={`/projects/${projectId}/preview/`} sandbox="allow-scripts" class="preview__frame"></iframe>
	</div>
{:else if runtime.previewUrl}
	<div class="preview">
		<div class="preview__bar">
			<span class="preview__status" data-phase={runtime.phase}>{PHASE_LABEL[runtime.phase]()}</span>
			{#if runtime.running}<span class="preview__running">{m.project_koda_running({ command: runtime.running })}</span>{/if}
			<span class="preview__spacer"></span>
			<Button variant="text" size="s" onclick={reload}>{m.project_reload_preview()}</Button>
			<Button variant="text" size="s" href={runtime.previewUrl} target="_blank" rel="noopener">{m.project_open_new_tab()}</Button>
		</div>
		<iframe bind:this={frame} title={m.project_preview_title()} src={runtime.previewUrl} class="preview__frame" allow="cross-origin-isolated; clipboard-write"></iframe>
	</div>
{:else}
	<div class="preview preview--status">
		{#if runtime.phase === 'failed'}
			<AgentShape agent="coding" size={72} />
		{:else if runtime.phase === 'waiting'}
			<AgentShape agent="coding" size={72} working />
		{:else}
			<LoadingIndicator size={64} label={PHASE_LABEL[runtime.phase]()} />
		{/if}
		<h3 class="preview__title">{PHASE_LABEL[runtime.phase]()}</h3>
		{#if runtime.phase === 'waiting'}
			<p class="preview__note">{m.project_waiting_body()}</p>
		{:else if runtime.phase === 'failed'}
			<p class="preview__note">{runtime.error}</p>
			{#if runtime.handedToKoda}<p class="preview__note">{m.project_check_failed_koda()}</p>{/if}
			<Button variant="tonal" size="s" onclick={() => runtime.restart()}>{m.project_restart()}</Button>
		{/if}
		{#if lastLines}<pre class="preview__log">{lastLines}</pre>{/if}
	</div>
{/if}

<style>
	.preview {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.preview--status {
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 24px;
		text-align: center;
	}
	.preview__bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 12px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.8125rem;
	}
	.preview__status {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-weight: 600;
	}
	.preview__running {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.preview__spacer {
		flex: 1;
	}
	.preview__frame {
		flex: 1;
		width: 100%;
		min-height: 0;
		border: 0;
		background: white;
	}
	.preview__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.preview__note {
		max-width: 46ch;
		margin: 0;
		padding: 0 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.preview > .preview__note {
		padding: 8px 12px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		max-width: none;
	}
	.preview__log {
		width: min(100%, 640px);
		max-height: 9.5em;
		margin: 8px 0 0;
		padding: 10px 12px;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.5;
		text-align: left;
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
