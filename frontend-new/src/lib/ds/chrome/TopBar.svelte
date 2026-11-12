<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Home → Top Bar (u8pcLQ): the page's name, then what it's about in mono after a slash
	// ("Home / Wed, 21 Oct 2026 · 08:14"), 10 apart; actions at the end. It fills AppShell's topbar
	// slot, which draws the 52px bar, its 24px sides and its bottom stroke.
	type Props = { title: string; meta?: string; actions?: Snippet };
	let { title, meta, actions }: Props = $props();
</script>

<div class="flex min-w-0 items-center justify-between gap-3">
	<p class="flex min-w-0 items-center gap-2.5">
		<span class="shrink-0 text-ui font-medium text-text">{title}</span>
		{#if meta}
			<span aria-hidden="true" class="font-mono text-label text-text-3">/</span>
			<span class="truncate font-mono text-label text-text-3">{meta}</span>
		{/if}
	</p>
	{#if actions}<div class="flex shrink-0 items-center gap-2">{@render actions()}</div>{/if}
</div>
