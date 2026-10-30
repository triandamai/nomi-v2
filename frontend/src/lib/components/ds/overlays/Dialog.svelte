<script lang="ts">
	import type { Snippet } from 'svelte';
	import DialogPanel from './DialogPanel.svelte';

	// Twin of frontend-new's Dialog: a native M3 dialog on the scrim, in the top layer, with focus
	// kept inside until it closes (Escape, or its own buttons via `open`). `wide` where it holds a
	// picker (moving a chat); `inline` shows the panel in place, open (the gallery).
	type Props = { open?: boolean; inline?: boolean; title: string; description?: string; wide?: boolean; children?: Snippet; actions?: Snippet; onclose?: () => void };
	let { open = $bindable(false), inline = false, title, description, wide = false, children, actions, onclose }: Props = $props();

	const id = $props.id();
	let wasOpen = false;

	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
	}
</script>

{#if inline}
	<DialogPanel {id} {title} {description} {wide} {children} {actions} />
{:else if open}
	<dialog
		use:modal
		aria-labelledby="{id}-title"
		aria-describedby={description ? `${id}-description` : undefined}
		onclose={() => (open = false)}
		class="ds-dialog"
	>
		<DialogPanel {id} {title} {description} {wide} {children} {actions} />
	</dialog>
{/if}

<style>
	.ds-dialog {
		margin: auto;
		max-width: calc(100% - 32px);
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
	}
	.ds-dialog::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 32%, transparent);
	}
</style>
