<script lang="ts">
	import type { Snippet } from 'svelte';
	import SheetHeader from '../mobile/SheetHeader.svelte';

	// Flow / Mobile Chat → Chat sheet (Manage chat & threads), Project menu (Mobile Build): a phone
	// uses a sheet where a desktop uses a menu or popover. It rises from the bottom over the scrim:
	// the surface with a border-strong stroke on top and 22 corners there, the Mobile Sheet Header
	// (grabber, title, what it's about, close), then its rows, with room for the home indicator.
	// A native modal dialog, so focus stays inside and Escape closes it; so does a tap on the scrim.
	// The header's close is drawn 30 round (Mobile / Sheet Header, the old app's twin) and takes a
	// 44 target here (UX contract phone-chat-flows: every tappable element is 44).
	// `header` replaces the Mobile Sheet Header where the pen draws another (a file's sheet, BhXcx:
	// the name in mono and the reason in danger); it's given the sheet's close.
	type Props = { open?: boolean; title: string; subtitle?: string; header?: Snippet<[() => void]>; children?: Snippet; onclose?: () => void };
	let { open = $bindable(false), title, subtitle, header, children, onclose }: Props = $props();

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
		aria-label={title}
		onclose={() => (open = false)}
		onclick={(event) => {
			// The dialog itself is the scrim: a tap outside the sheet lands on it.
			if (event.target === event.currentTarget) open = false;
		}}
		class="fixed inset-x-0 top-auto bottom-0 m-0 h-auto max-h-dvh w-full max-w-none bg-transparent p-0 text-text backdrop:bg-overlay"
	>
		<div data-sheet class="sheet flex max-h-dvh flex-col overflow-y-auto rounded-t-sheet border-t border-border-strong bg-surface pb-safe">
			{#if header}{@render header(() => (open = false))}{:else}<SheetHeader {title} {subtitle} onclose={() => (open = false)} />{/if}
			<div class="flex flex-col pb-2">{@render children?.()}</div>
		</div>
	</dialog>
{/if}

<style>
	/* The close's 44 target around its 30 circle: 7 on each side (the twin draws only the circle). */
	.sheet > :global(:first-child button) {
		position: relative;
	}
	.sheet > :global(:first-child button::after) {
		content: '';
		position: absolute;
		inset: -7px;
	}
</style>
