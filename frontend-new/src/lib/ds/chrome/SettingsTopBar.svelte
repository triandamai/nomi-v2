<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / * → Top Bar: where the person is in settings ("Settings › Profile"), the
	// section quiet, the page named, 10 apart; the page's actions at the end. It fills AppShell's
	// topbar slot, which draws the 52px bar, its 24px sides and its bottom stroke.
	type Props = { section: string; page: string; actions?: Snippet };
	let { section, page, actions }: Props = $props();
</script>

<div class="flex min-w-0 items-center justify-between gap-3">
	<p class="flex min-w-0 items-center gap-2.5">
		<span class="shrink-0 text-ui text-text-3">{section}</span>
		<span aria-hidden="true" class="font-mono text-label text-text-3">›</span>
		<span class="truncate text-ui font-medium text-text" aria-current="page">{page}</span>
	</p>
	{#if actions}<div class="flex shrink-0 items-center gap-2.5">{@render actions()}</div>{/if}
</div>
