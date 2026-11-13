<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Models & providers (RbTgo) → Connect Row: room for another provider, outlined,
	// [12,14]: the plus, what can be connected in mono, a few to start with as pills, and All.
	type Props = { title: string; detail: string; suggestions: { id: string; name: string }[]; all: string; onpick?: (id: string) => void; onall?: () => void };
	let { title, detail, suggestions, all, onpick, onall }: Props = $props();
</script>

<div class="flex flex-wrap items-center gap-2.5 rounded-md px-3.5 py-3 outline -outline-offset-1 outline-border">
	<button type="button" onclick={onall} class="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-sm text-left outline-offset-2">
		<span class="inline-flex size-8 shrink-0 items-center justify-center rounded-sm text-text outline -outline-offset-1 outline-border-strong"><Icon icon={Plus} /></span>
		<span class="flex min-w-0 flex-col gap-0.5">
			<span class="text-ui font-medium text-text">{title}</span>
			<span class="truncate font-mono text-label text-text-3">{detail}</span>
		</span>
	</button>
	<span class="flex flex-wrap items-center gap-2.5">
		{#each suggestions as suggestion (suggestion.id)}
			<button type="button" onclick={() => onpick?.(suggestion.id)} class="h-7 cursor-pointer rounded-full border border-border-strong px-2.25 text-caption text-text-2 outline-offset-2 hover:bg-surface-2 hover:text-text">
				{suggestion.name}
			</button>
		{/each}
		<button type="button" onclick={onall} class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text">{all}</button>
	</span>
</div>
