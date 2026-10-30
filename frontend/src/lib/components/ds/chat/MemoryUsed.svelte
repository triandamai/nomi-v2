<script lang="ts">
	import { Bookmark } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Memory / Used: under an answer that leaned on something remembered. Memory
	// is never silent.
	type Props = { fact: string; onmanage?: () => void };
	let { fact, onmanage }: Props = $props();
</script>

<p class="flex flex-wrap items-center gap-1.5">
	<span class="inline-flex items-center gap-1.5 text-caption text-text-3"><Icon icon={Bookmark} size={12} />{m.memory_used({ fact })}</span>
	{#if onmanage}
		<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onmanage}>{m.memory_manage()}</button>
	{/if}
</p>
