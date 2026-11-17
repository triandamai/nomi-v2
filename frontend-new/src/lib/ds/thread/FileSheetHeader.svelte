<script lang="ts">
	import { X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Attachments, step 10 (BhXcx, its Sheet Header r9vDW): the top of a file's
	// sheet. The Mobile Sheet Header's grabber, padding and round close, with the file's name in mono
	// (15, semibold) and what happened to it in danger mono under it. BottomSheet takes it as its
	// `header`.
	type Props = { name: string; reason: string; onclose: () => void };
	let { name, reason, onclose }: Props = $props();
</script>

<div class="flex w-full flex-col">
	<span aria-hidden="true" class="flex h-5 items-center justify-center"><span class="h-1.25 w-9 rounded-full bg-border-strong"></span></span>
	<div class="flex items-center gap-3 px-4 pt-0.5 pb-3">
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<h2 class="truncate font-mono text-body font-semibold text-text">{name}</h2>
			<span class="truncate font-mono text-label text-danger">{reason}</span>
		</span>
		<button type="button" aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1" onclick={onclose}>
			<Icon icon={X} />
		</button>
	</div>
</div>
