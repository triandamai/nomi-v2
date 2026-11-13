<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Design System / Projects → Project / Card / Loading (q7MGP): a card's place while the list
	// loads, at its exact size, its stroke quieter (border, not border-strong). Still blocks where the
	// tile, name, line, status zone, footer and action go; no shimmer (honest states).
	type Props = { label?: string };
	let { label = m.projects_loading() }: Props = $props();
</script>

<div role="status" aria-busy="true" aria-label={label} class="flex h-48 w-96 max-w-full shrink-0 flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1 outline-border">
	<div class="flex h-9 items-center gap-3" aria-hidden="true">
		<span class="size-9 shrink-0 rounded-sm bg-surface-2"></span>
		<span class="flex min-w-0 flex-1 flex-col gap-1.5">
			<span class="h-2.5 w-37.5 rounded-2xs bg-surface-2"></span>
			<span class="h-2 w-57.5 max-w-full rounded-2xs bg-surface-2"></span>
		</span>
	</div>
	<span class="h-14.5 shrink-0 rounded-sm bg-bg" aria-hidden="true"></span>
	<div class="flex items-center justify-between border-t border-border pt-2.75" aria-hidden="true">
		<span class="h-2 w-30 rounded-2xs bg-surface-2"></span>
		<span class="h-7.5 w-16 rounded-sm bg-surface-2"></span>
	</div>
</div>
