<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Design System / Projects → Project / Card / Loading (q7MGP): a card's place while the list
	// loads, at its exact size, its stroke quieter (border, not border-strong). Still blocks where the
	// tile, name, line, status zone, footer and action go; a shimmer sweeps them (reduced motion keeps them still).
	// `phone` is Mobile / Project Card / Loading (t3QMj; Flow / Mobile Projects — Loading V6bMR): the
	// column's width, 194 tall, 14 corners and the phone card's border-strong stroke; a 40 tile with
	// the name's bars (12 and 10, 4 corners, 3 apart), the status box on the page's ground ([10,12],
	// 10 corners) holding two bars 6 apart, and the footer's bar beside an 84 pill.
	type Props = { label?: string; phone?: boolean };
	let { label = m.projects_loading(), phone = false }: Props = $props();
</script>

{#if phone}
	<div role="status" aria-busy="true" aria-label={label} class="flex w-full max-w-full shrink-0 flex-col gap-3 rounded-xl bg-surface p-4 outline -outline-offset-1 outline-border-strong">
		<div class="flex h-10 items-center gap-3" aria-hidden="true">
			<span class="size-10 shrink-0 rounded-md shimmer"></span>
			<span class="flex min-w-0 flex-1 flex-col gap-0.75">
				<span class="h-3 w-37.5 rounded-xs shimmer"></span>
				<span class="h-2.5 w-50 max-w-full rounded-xs shimmer"></span>
			</span>
		</div>
		<div class="flex flex-col gap-1.5 rounded-md bg-bg px-3 py-2.5" aria-hidden="true">
			<span class="h-2.5 w-22.5 rounded-xs shimmer"></span>
			<span class="h-2.5 w-55 max-w-full rounded-xs shimmer"></span>
		</div>
		<div class="flex items-center justify-between border-t border-border pt-2.75" aria-hidden="true">
			<span class="h-2.5 w-30 rounded-xs shimmer"></span>
			<span class="h-10 w-21 rounded-full shimmer"></span>
		</div>
	</div>
{:else}
<div role="status" aria-busy="true" aria-label={label} class="flex h-48 w-96 max-w-full shrink-0 flex-col gap-3 rounded-md bg-surface p-4 outline -outline-offset-1 outline-border">
	<div class="flex h-9 items-center gap-3" aria-hidden="true">
		<span class="size-9 shrink-0 rounded-sm shimmer"></span>
		<span class="flex min-w-0 flex-1 flex-col gap-1.5">
			<span class="h-2.5 w-37.5 rounded-2xs shimmer"></span>
			<span class="h-2 w-57.5 max-w-full rounded-2xs shimmer"></span>
		</span>
	</div>
	<span class="h-14.5 shrink-0 rounded-sm bg-bg" aria-hidden="true"></span>
	<div class="flex items-center justify-between border-t border-border pt-2.75" aria-hidden="true">
		<span class="h-2 w-30 rounded-2xs shimmer"></span>
		<span class="h-7.5 w-16 rounded-sm shimmer"></span>
	</div>
</div>
{/if}
