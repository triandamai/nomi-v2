<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile Build → the sheets' lists (Deployments pefkG, Database jyEKQ, Code C0kOpT, People
	// OgP6t): rows in one card, a 14-round border, each row divided from the next. A list.
	type Props = { label?: string; children: Snippet };
	let { label, children }: Props = $props();
</script>

<ul aria-label={label} class="flex w-full flex-col overflow-hidden rounded-xl outline -outline-offset-1 outline-border">
	{@render children()}
</ul>
