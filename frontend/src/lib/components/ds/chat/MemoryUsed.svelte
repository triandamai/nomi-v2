<script lang="ts">
	import { Bookmark, ChevronDown } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Memory / Used: under an answer that leaned on something remembered. Memory
	// is never silent. An answer that used several folds them into one line ("From memory: Sari is
	// vegetarian and 3 more") that opens to list the rest, so a reply isn't buried under its memories.
	type Props = { fact: string; more?: string[]; onmanage?: () => void };
	let { fact, more = [], onmanage }: Props = $props();
	const listId = $props.id();
	let open = $state(false);
</script>

<div>
	<p class="flex flex-wrap items-center gap-1.5">
		{#if more.length > 0}
			<button
				type="button"
				class="inline-flex cursor-pointer items-center gap-1.5 rounded-xs text-left text-caption text-text-3 outline-offset-1 hover:text-text-2 pointer-coarse:min-h-11"
				aria-expanded={open}
				aria-controls={listId}
				onclick={() => (open = !open)}
			>
				<Icon icon={Bookmark} size={12} />{m.memory_used_folded({ fact, count: more.length })}<span class={['inline-flex', open && 'rotate-180']}><Icon icon={ChevronDown} size={12} /></span>
			</button>
		{:else}
			<span class="inline-flex items-center gap-1.5 text-caption text-text-3"><Icon icon={Bookmark} size={12} />{m.memory_used({ fact })}</span>
		{/if}
		{#if onmanage}
			<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onmanage}>{m.memory_manage()}</button>
		{/if}
	</p>
	{#if open}
		<ul id={listId} class="mt-1 flex flex-col gap-1 pl-4.5">
			{#each more as other (other)}
				<li class="text-caption text-text-3">{other}</li>
			{/each}
		</ul>
	{/if}
</div>
