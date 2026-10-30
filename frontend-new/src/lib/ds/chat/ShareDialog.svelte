<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import SharePanel from './SharePanel.svelte';

	// Chat responses → Share Dialog: the share panel as a native modal on the nm-overlay scrim. It
	// closes with its own button or Escape. `inline`: shown in place, open, as the design sheet draws it.
	type Props = Omit<ComponentProps<typeof SharePanel>, 'titleId' | 'onclose'> & { open?: boolean; inline?: boolean; onclose?: () => void };
	let { open = $bindable(false), inline = false, onclose, ...panel }: Props = $props();

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
	<section aria-labelledby="{id}-title" class="flex w-115 max-w-full flex-col overflow-hidden rounded-md bg-surface text-text shadow-float outline -outline-offset-1 outline-border-strong">
		<SharePanel {...panel} titleId="{id}-title" onclose={() => onclose?.()} />
	</section>
{:else if open}
	<dialog
		use:modal
		aria-labelledby="{id}-title"
		onclose={() => (open = false)}
		class="m-auto flex w-115 max-w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong p-0 text-text shadow-float backdrop:bg-overlay"
	>
		<SharePanel {...panel} titleId="{id}-title" onclose={() => (open = false)} />
	</dialog>
{/if}
