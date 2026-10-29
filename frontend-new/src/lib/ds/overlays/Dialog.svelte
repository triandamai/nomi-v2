<script lang="ts">
	import type { Snippet } from 'svelte';

	// Design System / Nomi → Overlays: a native modal dialog on the nm-overlay scrim, in the top
	// layer, with focus kept inside until it closes (Escape, or its own buttons via `open`).
	// 420 wide; `wide` (440) where the pen draws a picker in it (moving a chat).
	type Props = { open?: boolean; title: string; description?: string; wide?: boolean; children?: Snippet; actions?: Snippet; onclose?: () => void };
	let { open = $bindable(false), title, description, wide = false, children, actions, onclose }: Props = $props();

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

{#if open}
	<dialog
		use:modal
		aria-labelledby="{id}-title"
		aria-describedby={description ? `${id}-description` : undefined}
		onclose={() => (open = false)}
		class={['m-auto flex max-w-full flex-col overflow-hidden rounded-md border border-border-strong bg-surface p-0 text-text shadow-float backdrop:bg-overlay', wide ? 'w-110' : 'w-105']}
	>
		<div class="flex flex-col gap-2 p-5">
			<h2 id="{id}-title" class="text-body font-medium text-text">{title}</h2>
			{#if description}<p id="{id}-description" class="text-ui leading-normal text-text-2">{description}</p>{/if}
			{@render children?.()}
		</div>
		{#if actions}
			<div class="flex flex-wrap items-center justify-end gap-2 border-t border-border px-5 py-3">{@render actions()}</div>
		{/if}
	</dialog>
{/if}
