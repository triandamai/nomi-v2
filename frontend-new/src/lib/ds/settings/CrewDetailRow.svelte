<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / Crew (vVUeh) → Row: one fact about how a member works, [7,0] over the row's
	// bottom stroke (so 6 below), its name (13/500) over what it means, 12 from its control.
	type Props = { title: string; description?: string; control?: Snippet };
	let { title, description, control }: Props = $props();
</script>

<div data-part="row" class="flex min-h-12 items-center gap-3 border-b border-border pt-1.75 pb-1.5">
	<div class="flex min-w-0 flex-1 flex-col gap-px">
		<span class="text-ui font-medium text-text">{title}</span>
		{#if description}<span class="text-caption text-text-3">{description}</span>{/if}
	</div>
	{#if control}<div class="flex shrink-0 items-center gap-2">{@render control()}</div>{/if}
</div>
