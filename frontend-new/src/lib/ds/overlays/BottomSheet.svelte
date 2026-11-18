<script lang="ts">
	import type { Snippet } from 'svelte';
	import SheetHeader from '../mobile/SheetHeader.svelte';

	// Flow / Mobile Chat → Chat sheet (Manage chat & threads), Project menu (Mobile Build): a phone
	// uses a sheet where a desktop uses a menu or popover. It rises from the bottom over the scrim:
	// the surface with a border-strong stroke on top and 22 corners there, the Mobile Sheet Header
	// (grabber, title, what it's about, close), then its rows, with room for the home indicator.
	// A native modal dialog, so focus stays inside and Escape closes it; so does a tap on the scrim.
	// It opens with the sheet itself focused, not its close (Message controls rQ4w3: no ring on Close
	// after a long press): Tab goes on to the close and the rows.
	type Props = { open?: boolean; title: string; subtitle?: string; children?: Snippet; onclose?: () => void };
	let { open = $bindable(false), title, subtitle, children, onclose }: Props = $props();

	let wasOpen = false;
	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
		if (!dialog.querySelector('[autofocus]')) dialog.focus();
	}
</script>

{#if open}
	<dialog
		use:modal
		aria-label={title}
		tabindex="-1"
		onclose={() => (open = false)}
		onclick={(event) => {
			// The dialog itself is the scrim: a tap outside the sheet lands on it.
			if (event.target === event.currentTarget) open = false;
		}}
		class="fixed inset-x-0 top-auto bottom-0 m-0 h-auto max-h-dvh w-full max-w-none bg-transparent p-0 text-text outline-none backdrop:bg-overlay"
	>
		<div data-sheet class="flex max-h-dvh flex-col overflow-y-auto rounded-t-sheet border-t border-border-strong bg-surface pb-safe">
			<SheetHeader {title} {subtitle} onclose={() => (open = false)} />
			<div class="flex flex-col pb-2">{@render children?.()}</div>
		</div>
	</dialog>
{/if}
