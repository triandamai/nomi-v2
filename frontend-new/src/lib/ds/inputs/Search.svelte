<script lang="ts">
	import { Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Kbd from '../actions/Kbd.svelte';

	// Design System / Nomi → Search: a quiet 30px box (nm-border, not -strong) with its shortcut.
	// `plain` is the settings nav's: 10px in, the shortcut as plain mono text instead of a key.
	// `phone` is a phone list's (Flow / Mobile Chat → Chats → Search): 38 tall on surface-2 with no
	// stroke, 12 corners, [0,12], the 16 glyph and 16pt text (so iOS doesn't zoom), 8 apart.
	type Props = { placeholder: string; value?: string; shortcut?: string; label?: string; plain?: boolean; phone?: boolean; oninput?: (value: string) => void };
	let { placeholder, value = $bindable(''), shortcut, label, plain = false, phone = false, oninput }: Props = $props();
</script>

<label
	class={[
		'flex w-full items-center gap-2 text-text-3 focus-within:border-accent focus-within:shadow-focus',
		phone ? 'h-9.5 rounded-lg border border-transparent bg-surface-2 px-2.75' : ['h-7.5 rounded-sm border border-border', plain ? 'px-2.25' : 'px-2.75'],
	]}
>
	<Icon icon={SearchIcon} size={phone ? 16 : 14} />
	<input
		type="search"
		{placeholder}
		aria-label={label ?? placeholder}
		bind:value
		oninput={() => oninput?.(value)}
		class={['min-w-0 flex-1 bg-transparent text-text outline-none placeholder:text-text-3', phone ? 'text-phone' : 'text-ui']}
	/>
	{#if shortcut && plain}<span class="font-mono text-label text-text-3">{shortcut}</span>{:else if shortcut}<Kbd keys={shortcut} />{/if}
</label>
