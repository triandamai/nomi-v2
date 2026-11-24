<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile — Home, Inbox & You, step 2 (fpYK0) → Today: one thing on the day's timeline, its
	// time in mono, the title at 15 and its kind in mono under it, where it stands on the right (the
	// accent once done), and the row's tools. Done rows step back to text-2.
	type Props = { time: string; title: string; meta?: string; state?: string; done?: boolean; tools?: Snippet };
	let { time, title, meta, state: standing, done = false, tools }: Props = $props();
</script>

<li class="flex items-start gap-3 border-b border-border py-2.5">
	<span class={['shrink-0 pt-px font-mono text-ui', done ? 'text-text-3' : 'text-text']}>{time}</span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class={['text-body', done ? 'text-text-2' : 'text-text']}>{title}</span>
		{#if meta}<span class="truncate font-mono text-label text-text-3">{meta}</span>{/if}
	</span>
	{#if standing}<span class={['shrink-0 pt-0.5 font-mono text-label', done ? 'text-accent' : 'text-text-3']}>{standing}</span>{/if}
	{#if tools}<span class="-my-1 flex shrink-0 items-center">{@render tools()}</span>{/if}
</li>
