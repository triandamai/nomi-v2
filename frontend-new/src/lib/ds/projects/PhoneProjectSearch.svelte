<script lang="ts">
	import { ArrowDownUp, Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Project Search (Flow / Mobile Projects — Find & filter): [0,16,10,16], 8 apart, the
	// field 40 tall on surface-2 with 12 corners ([0,12], the 16 glyph, the text at 16pt so iOS
	// doesn't zoom) and Sort (40, surface-2, 12 corners, arrow-down-up) opening the sort sheet.
	type Props = { placeholder: string; value?: string; sortLabel: string; onsort?: () => void };
	let { placeholder, value = $bindable(''), sortLabel, onsort }: Props = $props();
</script>

<div class="flex items-center gap-2 px-4 pb-2.5">
	<label class="flex h-10 min-w-0 flex-1 items-center gap-2 rounded-lg border border-transparent bg-surface-2 px-2.75 text-text-3 focus-within:border-accent focus-within:shadow-focus">
		<Icon icon={SearchIcon} size={16} />
		<input type="search" {placeholder} aria-label={placeholder} bind:value class="min-w-0 flex-1 bg-transparent text-phone text-text outline-none placeholder:text-text-3" />
	</label>
	{#if onsort}
		<button type="button" aria-label={sortLabel} class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-lg bg-surface-2 text-text-2 outline-offset-2" onclick={onsort}>
			<Icon icon={ArrowDownUp} size={18} />
		</button>
	{/if}
</div>
