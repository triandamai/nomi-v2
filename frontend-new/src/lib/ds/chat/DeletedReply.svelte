<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Deleted Reply: what's left where a reply was, so the thread still makes
	// sense. It can come back, or be asked again.
	type Props = { name: string; onundo?: () => void; onregenerate?: () => void };
	let { name, onundo, onregenerate }: Props = $props();
</script>

<p class="flex w-full flex-wrap items-center gap-x-2 gap-y-1 rounded-sm border border-border px-3 py-2.5">
	<span class="inline-flex shrink-0 text-text-3"><Icon icon={Trash2} size={14} /></span>
	<span class="min-w-0 flex-1 basis-40 text-caption text-text-3 italic">{m.deleted_reply({ name })}</span>
	<span class="ml-auto flex items-center gap-2">
		{#if onundo}
			<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onundo}>{m.action_undo()}</button>
		{/if}
		{#if onregenerate}
			<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onregenerate}>{m.reply_regenerate()}</button>
		{/if}
	</span>
</p>
