<script lang="ts">
	import { BookmarkPlus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Memory / Saved: something new remembered, said where it happened, with edit
	// and undo right there.
	type Props = { fact: string; onedit?: () => void; onundo?: () => void };
	let { fact, onedit, onundo }: Props = $props();
</script>

<p role="status" class="flex w-full flex-wrap items-center gap-x-2 gap-y-1 rounded-sm bg-surface px-3 py-2">
	<span class="inline-flex shrink-0 text-accent"><Icon icon={BookmarkPlus} size={14} /></span>
	<span class="min-w-0 flex-1 basis-48 text-caption text-text-2">{m.memory_saved({ fact })}</span>
	<span class="ml-auto flex items-center gap-2">
		{#if onedit}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onedit}>{m.action_edit()}</button>{/if}
		{#if onundo}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onundo}>{m.action_undo()}</button>{/if}
	</span>
</p>
