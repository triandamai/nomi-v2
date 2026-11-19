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
	// The header's close is drawn 30 round (Mobile / Sheet Header, the old app's twin) and takes a
	// 44 target here (UX contract phone-chat-flows: every tappable element is 44).
	// `header` replaces the Mobile Sheet Header where the pen draws another (a file's sheet, BhXcx:
	// the name in mono and the reason in danger); it's given the sheet's close.
	// `lifted` is what the sheet is about, held over the scrim just above it (Message controls nzT7f:
	// your message, right-aligned, [0,16,12,16]); a tap beside it closes the sheet like the scrim.
	type Props = { open?: boolean; title: string; subtitle?: string; header?: Snippet<[() => void]>; children?: Snippet; lifted?: Snippet; onclose?: () => void };
	let { open = $bindable(false), title, subtitle, header, children, lifted, onclose }: Props = $props();

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
		{#if lifted}
			<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
			<div data-lifted class="flex justify-end px-4 pb-3" onclick={(event) => event.target === event.currentTarget && (open = false)}>{@render lifted()}</div>
		{/if}
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
