<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / Models & providers / Connect (voUxB) → Dialog: the providers to pick from in a
	// 320 column on bg beside the pane that connects one, in one 900 × 620 box on the surface,
	// border-strong, the one shadow. On a phone the list sits over the pane.
	type Props = { list: Snippet; pane: Snippet };
	let { list, pane }: Props = $props();
</script>

<div class="flex h-155 max-h-dvh w-225 max-w-full flex-col overflow-hidden rounded-md bg-surface shadow-float outline -outline-offset-1 outline-border-strong md:flex-row">
	<div class="flex max-h-60 min-h-0 shrink-0 flex-col border-b border-border bg-bg md:max-h-none md:w-80 md:border-r md:border-b-0">{@render list()}</div>
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">{@render pane()}</div>
</div>
