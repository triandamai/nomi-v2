<script lang="ts">
	// The inside of a project tool sheet: a title bar that stays put, and the tool below it.
	import { m } from '$lib/paraglide/messages';
	import type { Snippet } from 'svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';

	let {
		title,
		subtitle,
		onclose,
		actions,
		children,
	}: { title: string; subtitle?: string; onclose: () => void; actions?: Snippet; children: Snippet } = $props();
</script>

<div class="frame">
	<header class="frame__bar">
		<div class="frame__titles">
			<h2 class="frame__title">{title}</h2>
			{#if subtitle}<p class="frame__subtitle">{subtitle}</p>{/if}
		</div>
		{#if actions}<div class="frame__actions">{@render actions()}</div>{/if}
		<IconButton onclick={onclose} aria-label={m.pj_close()} title={m.pj_close()}><IconClose size={20} /></IconButton>
	</header>
	<div class="frame__body">
		<div class="frame__inner">{@render children()}</div>
	</div>
</div>

<style>
	.frame {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-height: 0;
	}
	.frame__bar {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 0 12px 10px 20px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.frame__titles {
		flex: 1;
		min-width: 0;
	}
	.frame__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.1875rem;
		font-weight: 700;
		letter-spacing: -0.01em;
	}
	.frame__subtitle {
		margin: 2px 0 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.frame__actions {
		display: flex;
		flex: none;
		align-items: center;
		gap: 8px;
	}
	.frame__body {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
	}
	.frame__inner {
		box-sizing: border-box;
		max-width: 1080px;
		margin: 0 auto;
		padding: 16px clamp(14px, 3vw, 28px) calc(28px + env(safe-area-inset-bottom));
	}
	@media (max-width: 600px) {
		.frame__bar {
			padding-left: 14px;
		}
	}
</style>
