<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Info, TriangleAlert } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';

	// An aside worth stopping for: a bordered box, never a coloured fill.
	let { tone = 'info', children }: { tone?: 'info' | 'warning'; children: Snippet } = $props();
</script>

<aside class="flex max-w-170 gap-3 rounded-md border border-border bg-surface px-4 py-3">
	<span class={['mt-0.5 inline-flex', tone === 'warning' ? 'text-warning' : 'text-text-3']}><Icon icon={tone === 'warning' ? TriangleAlert : Info} size={14} /></span>
	<div class="flex min-w-0 flex-col gap-2 text-ui leading-normal text-text-2">{@render children()}</div>
</aside>
