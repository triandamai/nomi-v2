<script lang="ts">
	import { ChevronsUpDown } from '@lucide/svelte';

	/** Which workspace's projects the Build group shows ("T Trian's space ↕"); opens the switcher. */
	let { name, onclick }: { name: string; onclick?: () => void } = $props();
</script>

<button type="button" {onclick} aria-haspopup="menu" class="inline-flex h-5 cursor-pointer items-center gap-1.25 rounded-xs bg-surface-2 pr-1.5 pl-0.75 outline-offset-1 hover:bg-border">
	<span aria-hidden="true" class="inline-flex size-3.5 items-center justify-center rounded-xs text-label font-semibold text-text">{name.charAt(0).toUpperCase()}</span>
	<span class="font-mono text-label text-text-2">{name}</span>
	<ChevronsUpDown size={10} strokeWidth={1.5} aria-hidden="true" class="text-text-3" />
</button>
