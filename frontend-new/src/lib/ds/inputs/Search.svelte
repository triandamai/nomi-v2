<script lang="ts">
	import { Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Kbd from '../actions/Kbd.svelte';

	// Design System / Nomi → Search: a quiet 30px box (nm-border, not -strong) with its shortcut.
	// `plain` is the settings nav's: 10px in, the shortcut as plain mono text instead of a key.
	type Props = { placeholder: string; value?: string; shortcut?: string; label?: string; plain?: boolean; oninput?: (value: string) => void };
	let { placeholder, value = $bindable(''), shortcut, label, plain = false, oninput }: Props = $props();
</script>

<label class={['flex h-7.5 w-full items-center gap-2 rounded-sm border border-border text-text-3 focus-within:border-accent focus-within:shadow-focus', plain ? 'px-2.5' : 'px-3']}>
	<Icon icon={SearchIcon} size={14} />
	<input
		type="search"
		{placeholder}
		aria-label={label ?? placeholder}
		bind:value
		oninput={() => oninput?.(value)}
		class="min-w-0 flex-1 bg-transparent text-ui text-text outline-none placeholder:text-text-3"
	/>
	{#if shortcut && plain}<span class="font-mono text-label text-text-3">{shortcut}</span>{:else if shortcut}<Kbd keys={shortcut} />{/if}
</label>
