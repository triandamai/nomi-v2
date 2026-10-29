<script lang="ts">
	import type { Snippet } from 'svelte';

	// A native modal for a panel that brings its own box (Share, Export): the nm-overlay scrim, the
	// top layer, focus kept inside, Escape closes. Named by the panel's heading (`labelledby`).
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
	<dialog use:modal aria-labelledby={labelledby} onclose={() => (open = false)} class="m-auto max-w-full bg-transparent p-0 text-text backdrop:bg-overlay">
		{@render children()}
	</dialog>
{/if}
