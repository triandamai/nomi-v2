<script lang="ts">
	import { ArrowRight, ChevronDown } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import RecentRow from '../navigation/RecentRow.svelte';

	// The sidebar's recent chats and projects, foldable, ending in a link to all chats.
	type Item = { href: string; kind: 'chat' | 'project'; title: string; when: string; active?: boolean };
	let { items, allHref, open = $bindable(true) }: { items: Item[]; allHref: string; open?: boolean } = $props();

	const id = $props.id();
</script>

<div class="flex flex-col gap-0.5">
	<button
		type="button"
		aria-expanded={open}
		aria-controls={id}
		onclick={() => (open = !open)}
		class="flex w-full cursor-pointer items-center justify-between rounded-xs px-2.5 pb-1.5 text-text-3 outline-offset-1 hover:text-text-2"
	>
		<span class="font-mono text-label tracking-label uppercase">{m.nav_recents()}</span>
		<span class={['inline-flex transition-transform', !open && '-rotate-90']}><Icon icon={ChevronDown} size={14} /></span>
	</button>
	{#if open}
		<div {id} class="flex flex-col gap-0.5">
			{#each items as item (item.href)}<RecentRow {...item} />{/each}
		</div>
	{/if}
	<!-- Folded or not, this stays: it's the way to every chat. -->
	<a href={allHref} class="flex h-7.5 items-center gap-1.5 rounded-sm px-2.5 text-ui text-text-3 outline-offset-1 hover:text-text-2">
		{m.nav_all_chats()}
		<Icon icon={ArrowRight} size={14} />
	</a>
</div>
