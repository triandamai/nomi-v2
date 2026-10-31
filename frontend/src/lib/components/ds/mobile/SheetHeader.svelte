<script lang="ts">
	import { X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Sheet Header: the top of a bottom sheet (phones use sheets, not popovers): the
	// grabber, the title at 17pt, what it's about, and a round close.
	type Props = { title: string; subtitle?: string; onclose: () => void };
	let { title, subtitle, onclose }: Props = $props();
</script>

<div class="flex w-full flex-col">
	<span aria-hidden="true" class="flex h-5 items-center justify-center"><span class="h-1.25 w-9 rounded-full bg-border-strong"></span></span>
	<div class="flex items-center gap-3 px-4 pt-0.5 pb-3">
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<h2 class="text-phone-title font-semibold text-text">{title}</h2>
			{#if subtitle}<span class="text-ui text-text-3">{subtitle}</span>{/if}
		</span>
		<button type="button" aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1" onclick={onclose}>
			<Icon icon={X} />
		</button>
	</div>
</div>
