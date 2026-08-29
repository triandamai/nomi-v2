<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon, { type IconName } from './Icon.svelte';

	let {
		open = $bindable(false),
		headline,
		icon,
		children,
		actions,
		class: extraClass = '',
	}: {
		open?: boolean;
		headline?: string;
		icon?: IconName;
		children: Snippet;
		actions?: Snippet;
		class?: string;
	} = $props();

	let dialogEl: HTMLDialogElement | undefined = $state();

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
		} else if (!open && dialogEl.open) {
			dialogEl.close();
		}
	});

	function handleClose() {
		// Fires on Escape (native) and on our own `close()` call above — the single place
		// that keeps `open` in sync with the dialog's actual state either way.
		open = false;
	}

	function handleDialogClick(event: MouseEvent) {
		// Native <dialog> has no built-in "click outside to dismiss" — clicking the
		// backdrop actually clicks the <dialog> element itself (its content sits in a
		// centered box inside it), while clicking real content clicks a descendant.
		if (event.target === dialogEl) open = false;
	}
</script>

<dialog bind:this={dialogEl} class="m3-dialog {extraClass}" onclose={handleClose} onclick={handleDialogClick}>
	<div class="m3-dialog__content">
		{#if icon}
			<div class="m3-dialog__icon"><Icon name={icon} size={24} /></div>
		{/if}
		{#if headline}
			<h2 class="md-headline-small m3-dialog__headline">{headline}</h2>
		{/if}
		<div class="m3-dialog__body">
			{@render children()}
		</div>
		{#if actions}
			<div class="m3-dialog__actions">
				{@render actions()}
			</div>
		{/if}
	</div>
</dialog>

<style>
	.m3-dialog {
		margin: auto;
		padding: 0;
		border: none;
		min-width: 280px;
		max-width: 560px;
		width: 90vw;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
	}

	.m3-dialog::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 32%, transparent);
	}

	.m3-dialog__content {
		display: flex;
		flex-direction: column;
		padding: 24px;
	}

	.m3-dialog__icon {
		color: var(--md-sys-color-secondary);
		margin-bottom: 16px;
	}

	.m3-dialog__headline {
		margin: 0 0 16px;
		color: var(--md-sys-color-on-surface);
	}

	.m3-dialog__body {
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		line-height: var(--md-sys-typescale-body-medium-line-height);
		color: var(--md-sys-color-on-surface-variant);
	}

	.m3-dialog__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 24px;
	}
</style>
