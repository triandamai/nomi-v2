<script lang="ts">
	import type { Snippet } from 'svelte';
	import PhoneBackBar from '../chrome/PhoneBackBar.svelte';

	// Flow / Mobile — You & Settings, steps 4, 5 and 15 (Change email, Enter the code, the key):
	// typing gets its own screen. The back bar, the title at 32 with what happens next under it, the
	// fields, and the one action at the foot, in reach and above the keyboard as the page shrinks to
	// it. A form, so the keyboard's return submits. Its inputs are 16pt here, so iOS doesn't zoom in
	// on them (the shared fields keep their own size elsewhere).
	type Props = {
		backLabel: string;
		onback: () => void;
		title: string;
		sub?: string;
		onsubmit: () => void;
		children: Snippet;
		action: Snippet;
	};
	let { backLabel, onback, title, sub, onsubmit, children, action }: Props = $props();
</script>

<form
	class="form flex min-h-full flex-col"
	onsubmit={(event) => {
		event.preventDefault();
		onsubmit();
	}}
>
	<PhoneBackBar label={backLabel} {onback} />
	<div class="flex flex-col gap-1 px-5 pt-1 pb-4">
		<h1 class="text-phone-large font-semibold tracking-phone-large text-text">{title}</h1>
		{#if sub}<p class="text-phone-sm leading-firm text-text-3">{sub}</p>{/if}
	</div>
	<div class="flex flex-col gap-4 px-4">{@render children()}</div>
	<!-- Return submits even with several fields (the action is the screen's own button). -->
	<button type="submit" hidden aria-hidden="true" tabindex="-1"></button>
	<span class="min-h-6 flex-1"></span>
	<div class="sticky bottom-0 flex flex-col gap-2 bg-bg px-4 pt-3 pb-4">{@render action()}</div>
</form>

<style>
	/* 16pt on a phone so iOS doesn't zoom into a field as it's focused. */
	.form :global(input) {
		font-size: var(--text-phone);
	}
</style>
