<script lang="ts">
	import { ExternalLink, Monitor, RotateCw, Smartphone } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';

	// The preview's address bar in the pane's top bar: whether it's live, where it lives, reload
	// and open in a new tab. The device switch is its own control beside it (PreviewDevices).
	type Props = {
		/** "Live", "Installing…": what the preview is doing. */
		status: string;
		/** Green dot: the preview is up. */
		running?: boolean;
		/** The address shown, without the protocol. */
		address: string;
		/** Where "open" goes. */
		href?: string | null;
		onreload?: () => void;
	};
	let { status, running = false, address, href = null, onreload }: Props = $props();
</script>

<div class="address">
	<span class="address__status"><span class="address__dot" data-running={running || undefined}></span><span class="address__label">{status}</span></span>
	<span class="address__url" title={address}>{address}</span>
	{#if onreload}
		<button type="button" class="address__btn" aria-label={m.project_reload_preview()} title={m.project_reload_preview()} onclick={onreload}><Icon icon={RotateCw} size={18} /></button>
	{/if}
	{#if href}
		<a class="address__btn" {href} target="_blank" rel="noopener" aria-label={m.project_open_new_tab()} title={m.project_open_new_tab()}><Icon icon={ExternalLink} size={18} /></a>
	{/if}
</div>

<style>
	.address {
		display: flex;
		flex: 1;
		min-width: 0;
		align-items: center;
		gap: 10px;
		min-height: 40px;
		padding: 0 4px 0 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	/* The status gives way well before the address (its label ellipsizes), so a tight bar keeps
	 * localhost:<port> readable and never spills over its neighbours. */
	.address__status {
		display: inline-flex;
		flex: 0 1000 auto;
		min-width: 0;
		overflow: hidden;
		align-items: center;
		gap: 8px;
		padding-right: 10px;
		border-right: 1px solid var(--md-sys-color-outline-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-transform: lowercase;
	}
	.address__label {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.address__dot {
		flex: none;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--md-sys-color-outline);
	}
	.address__dot[data-running] {
		background: var(--md-sys-color-primary);
	}
	/* Full width up to most of the bar (localhost:<port> always fits); a long published URL ellipsizes. */
	.address__url {
		flex: 1 0 auto;
		max-width: 70%;
		min-width: 0;
		overflow: hidden;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.address__btn {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.address__btn:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.address__btn:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	@media (pointer: coarse) {
		.address__btn {
			width: 44px;
			height: 44px;
		}
	}
</style>
