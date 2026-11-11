<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Inbox (BFoiZ) → Earlier Link: the entries from before today, folded away under their
	// name and count until asked; the chevron turns down when they're open.
	type Props = { label: string; count: number | string; expanded?: boolean; ontoggle?: (expanded: boolean) => void };
	let { label, count, expanded = $bindable(false), ontoggle }: Props = $props();

	function toggle() {
		expanded = !expanded;
		ontoggle?.(expanded);
	}
</script>

<button
	type="button"
	aria-expanded={expanded}
	onclick={toggle}
	class="flex cursor-pointer items-center gap-2 self-start rounded-xs py-0.5 font-mono text-label outline-offset-2 hover:text-text"
>
	<span class="tracking-label text-text-2 uppercase">{label}</span>
	<span class="text-text-3">{count}</span>
	<span class={['inline-flex text-text-3 transition-transform motion-reduce:transition-none', expanded && 'rotate-90']}><Icon icon={ChevronRight} size={12} /></span>
</button>
