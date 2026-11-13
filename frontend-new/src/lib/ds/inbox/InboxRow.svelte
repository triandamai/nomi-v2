<script lang="ts">
	import type { Snippet } from 'svelte';
	import SourceTile from './SourceTile.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Inbox (BFoiZ) → Today → Item: an entry in a day's list, a quiet 56px row ([10,4], the
	// bottom stroke): its time (the text colour while it's still ahead, quieter once it's past),
	// whose tile, the title with its mono tag, the line under it, and its tools at the end, 2 apart.
	// A finished one (a reminder done or cancelled) is dimmed.
	type Props = {
		time: string;
		agent: CrewId;
		title: string;
		tag?: string;
		caption?: string;
		upcoming?: boolean;
		finished?: boolean;
		tools?: Snippet;
		[attribute: `data-${string}`]: unknown;
	};
	let { time, agent, title, tag, caption, upcoming = false, finished = false, tools, ...rest }: Props = $props();
</script>

<li {...rest} class={['flex w-full items-center gap-3 border-b border-border px-1 pt-2.5 pb-2.25', finished && 'opacity-55']}>
	<span class={['w-14 shrink-0 font-mono text-label', upcoming ? 'text-text' : 'text-text-3']}>{time}</span>
	<SourceTile {agent} />
	<div class="flex min-w-0 flex-1 flex-col gap-0.75">
		<p class="flex min-w-0 items-center gap-2">
			<span class="min-w-0 truncate text-ui font-medium text-text">{title}</span>
			{#if tag}<span class="shrink-0 font-mono text-label text-text-3">{tag}</span>{/if}
		</p>
		{#if caption}<p class="truncate text-caption text-text-3">{caption}</p>{/if}
	</div>
	{#if tools}<div class="flex shrink-0 items-center gap-0.5">{@render tools()}</div>{/if}
</li>
