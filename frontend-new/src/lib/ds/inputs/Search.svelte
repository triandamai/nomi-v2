<script lang="ts">
	import { Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Kbd from '../actions/Kbd.svelte';

	// Design System / Nomi → Search: a quiet 30px box (nm-border, not -strong) with its shortcut.
	type Props = { placeholder: string; value?: string; shortcut?: string; label?: string; oninput?: (value: string) => void };
	let { placeholder, value = $bindable(''), shortcut, label, oninput }: Props = $props();
</script>

<label class="flex h-7.5 w-full items-center gap-2 rounded-sm border border-border px-3 text-text-3 focus-within:border-border-strong focus-within:ring-3 focus-within:ring-focus">
	<Icon icon={SearchIcon} size={14} />
	<input
		type="search"
		{placeholder}
		aria-label={label ?? placeholder}
		bind:value
		oninput={() => oninput?.(value)}
		class="min-w-0 flex-1 bg-transparent text-ui text-text outline-none placeholder:text-text-3"
	/>
	{#if shortcut}<Kbd keys={shortcut} />{/if}
</label>
