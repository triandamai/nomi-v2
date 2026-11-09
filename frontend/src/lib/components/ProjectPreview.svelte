<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import ProjectWaitScreen from '$lib/components/project/ProjectWaitScreen.svelte';
	import type { AnyRuntime } from '$lib/runtimeChooser';
	import PhoneFrame from '$lib/components/ds/build/PhoneFrame.svelte';
	import { PHASE_LABEL } from '$lib/components/project/previewPhase';

	// The project's live preview, from its dev server running in the browser (WebContainer),
	// with what it's doing while it gets there. Static projects, and browsers WebContainer can't
	// run in, get the plain file preview instead.
	let {
		projectId,
		runtime,
		stack = 'static',
		files = [],
		liveUrl = null,
		onShowCode,
		onUseBrowser,
		device = $bindable('desktop'),
	}: {
		projectId: string;
		runtime: AnyRuntime | null;
		/** What the project is built with; only a static one is shown straight from its files. */
		stack?: string;
		/** The project's file paths. */
		files?: string[];
		/** The published app's address: what "Open on your phone" sends. */
		liveUrl?: string | null;
		onShowCode?: () => void;
		/** Runs the project in the browser instead of the sandbox; offered only while the sandbox is the runtime. */
		onUseBrowser?: () => void;
		/** Desktop or phone width; the pane's top bar switches it. */
		device?: 'desktop' | 'phone';
	} = $props();

	/** Sends the published address to the person's own phone (share sheet, else the clipboard). */
	export async function openOnPhone() {
		if (!liveUrl) return;
		if (navigator.share) {
			try {
				await navigator.share({ url: liveUrl });
				return;
			} catch {
				// Dismissed: fall back to the clipboard.
			}
		}
		await navigator.clipboard.writeText(liveUrl);
	}

	// Showing the files as they are only works for a plain page with an index.html at the top;
	// anything else (or nothing yet) would just be a 404 in the frame.
	const hasIndex = $derived(files.includes('index.html'));
	const staticStack = $derived(stack === 'static');

	let frame = $state<HTMLIFrameElement | null>(null);

	// What the dev server is printing (installing, starting), passed down to the wait screen / terminal.
	const logText = $derived(runtime ? runtime.log.trimEnd().split('\n').slice(-500).join('\n') : '');

	/** Loads the running preview again. */
	export function reload() {
		if (frame && runtime?.previewUrl) frame.src = runtime.previewUrl;
	}
</script>

{#if !runtime || runtime.phase === 'unsupported'}
	{#if files.length === 0}
		<div class="preview preview--status">
			<AgentShape agent="coding" size={72} working />
			<h3 class="preview__title">{m.project_nothing_yet()}</h3>
			<p class="preview__note">{m.project_nothing_yet_body()}</p>
		</div>
	{:else if hasIndex && (staticStack || runtime)}
		<div class="preview">
			{#if runtime}<p class="preview__note">{m.project_unsupported_body()}</p>{/if}

			{#if device === 'phone'}
				<div class="preview__phone"><PhoneFrame src={`/projects/${projectId}/preview/`} title={m.project_preview_title()} /></div>
			{:else}
				<iframe title={m.project_preview_title()} src={`/projects/${projectId}/preview/`} sandbox="allow-scripts" class="preview__frame"></iframe>
			{/if}
		</div>
	{:else}
		<div class="preview preview--status">
			<AgentShape agent="coding" size={72} />
			<h3 class="preview__title">{m.project_live_needed()}</h3>
			<p class="preview__note">{staticStack ? m.project_no_index() : m.project_live_needed_body()}</p>
			<div class="preview__actions">
				{#if onShowCode}<Button variant="tonal" size="s" onclick={onShowCode}>{m.project_show_code()}</Button>{/if}
				{#if !staticStack}<Button variant="text" size="s" onclick={() => location.reload()}>{m.project_try_again()}</Button>{/if}
			</div>
		</div>
	{/if}
{:else if runtime.previewUrl}
	<div class="preview">
		{#if device === 'phone'}
			<div class="preview__phone"><PhoneFrame src={runtime.previewUrl} title={m.project_preview_title()} /></div>
		{:else}
			<iframe bind:this={frame} title={m.project_preview_title()} src={runtime.previewUrl} class="preview__frame" allow="cross-origin-isolated; clipboard-write"></iframe>
		{/if}
	</div>
{:else}
	<ProjectWaitScreen
		phase={runtime.phase}
		phaseLabel={PHASE_LABEL[runtime.phase]()}
		runningCommand={runtime.running}
		error={runtime.error}
		handedToKoda={runtime.handedToKoda}
		{logText}
		onRestart={() => runtime.restart()}
		onRetry={'retry' in runtime && typeof runtime.retry === 'function' ? () => (runtime as { retry: () => void }).retry() : undefined}
		{onUseBrowser}
	/>
{/if}

<style>
	.preview__phone {
		display: flex;
		flex: 1;
		min-height: 0;
		align-items: flex-start;
		justify-content: center;
		overflow: auto;
		padding: 24px 16px;
	}
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
		padding: 20px;
		text-align: center;
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
		font-size: 1.125rem;
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
	.preview__actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px;
	}
</style>
