<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Design System / Projects → Project / Card / Loading (q7MGP): a card's place while the list
	// loads, at its exact size, its stroke quieter (border, not border-strong). Still blocks where the
	// tile, name, line, status zone, footer and action go; no shimmer (honest states).
	// `phone` is Mobile / Project Card / Loading (Flow / Mobile Projects — Loading): the column's width,
	// 14 corners, a 40 tile, a 56 status box and a pill where the action goes.
	type Props = { label?: string; phone?: boolean };
	let { label = m.projects_loading(), phone = false }: Props = $props();
</script>

<div
	role="status"
	aria-busy="true"
	aria-label={label}
	class={['flex max-w-full shrink-0 flex-col gap-3 bg-surface p-4 outline -outline-offset-1 outline-border', phone ? 'w-full rounded-xl' : 'h-48 w-96 rounded-md']}
>
	<div class={['flex items-center gap-3', phone ? 'h-10' : 'h-9']} aria-hidden="true">
		<span class={['shrink-0 bg-surface-2', phone ? 'size-10 rounded-md' : 'size-9 rounded-sm']}></span>
		<span class="flex min-w-0 flex-1 flex-col gap-1.5">
			<span class="h-2.5 w-37.5 rounded-2xs bg-surface-2"></span>
			<span class="h-2 w-57.5 max-w-full rounded-2xs bg-surface-2"></span>
		</span>
	</div>
	<span class={['shrink-0 bg-bg', phone ? 'h-14 rounded-md' : 'h-14.5 rounded-sm']} aria-hidden="true"></span>
	<div class="flex items-center justify-between border-t border-border pt-2.75" aria-hidden="true">
		<span class="h-2 w-30 rounded-2xs bg-surface-2"></span>
		<span class={['bg-surface-2', phone ? 'h-10 w-21 rounded-full' : 'h-7.5 w-16 rounded-sm']}></span>
	</div>
</div>
