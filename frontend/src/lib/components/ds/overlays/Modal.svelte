<script lang="ts">
	import type { Snippet } from 'svelte';

	// A native modal for a panel that brings its own box (Share, Export): the scrim, the top layer,
	// focus kept inside, Escape closes. Named by the panel's heading (`labelledby`).
	type Props = { open?: boolean; labelledby: string; onclose?: () => void; children: Snippet };
	let { open = $bindable(false), labelledby, onclose, children }: Props = $props();

	let wasOpen = false;

	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
	}
</script>

{#if open}
	<dialog use:modal aria-labelledby={labelledby} onclose={() => (open = false)} class="ds-modal">
		{@render children()}
	</dialog>
{/if}

<style>
	.ds-modal {
		margin: auto;
		max-width: calc(100% - 32px);
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
	}
	.ds-modal::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 32%, transparent);
	}
</style>
