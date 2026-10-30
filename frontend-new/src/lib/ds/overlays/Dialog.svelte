<script lang="ts">
	import type { Snippet } from 'svelte';
	import DialogPanel from './DialogPanel.svelte';

	// Design System / Nomi → Overlays: a native modal dialog on the nm-overlay scrim, in the top
	// layer, with focus kept inside until it closes (Escape, or its own buttons via `open`).
	// 420 wide; `wide` (440) where the pen draws a picker in it (moving a chat). `inline` shows the
	// panel in place, open, as the design sheets draw it.
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
		class="m-auto max-w-full bg-transparent p-0 text-text backdrop:bg-overlay"
	>
		<DialogPanel {id} {title} {description} {wide} {children} {actions} />
	</dialog>
{/if}
