<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / Workspace (wZm9B) → Member: one person in a workspace, [11,0] over the row's
	// line and 12 apart: their 32px face (photo, or initial), the name (13/500) with "you" on the
	// person's own row, what we know in mono under it, and the controls on the right. A pending
	// invite is the same row with the address as its name.
	type Props = { name: string; initial: string; avatar?: string | null; you?: string; meta: string; controls?: Snippet };
	let { name, initial, avatar, you, meta, controls }: Props = $props();
</script>

<li class="flex min-h-13.75 flex-wrap items-center gap-3 border-b border-border pt-2.75 pb-2.5">
	{#if avatar}
		<img data-part="face" src={avatar} alt="" class="size-8 shrink-0 rounded-sm object-cover outline -outline-offset-1 outline-border-strong" />
	{:else}
		<span data-part="face" aria-hidden="true" class="inline-flex size-8 shrink-0 items-center justify-center rounded-sm bg-surface-2 font-mono text-ui text-text-2 outline -outline-offset-1 outline-border-strong">{initial}</span>
	{/if}
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<div class="flex min-w-0 items-center gap-1.5">
			<span class="truncate text-ui font-medium text-text">{name}</span>
			{#if you}<span class="inline-flex h-4.5 items-center rounded-xs bg-surface-2 px-1.25 font-mono text-label text-text-2">{you}</span>{/if}
		</div>
		<span class="truncate font-mono text-label text-text-3">{meta}</span>
	</div>
	{#if controls}<div class="flex shrink-0 items-center gap-3">{@render controls()}</div>{/if}
</li>
