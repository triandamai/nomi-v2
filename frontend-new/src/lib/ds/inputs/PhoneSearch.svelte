<script lang="ts">
	import { Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Start & answer → Chats → Search: a phone list's search, 38 tall on
	// surface-2 with no stroke, 12 corners, [0,12], the 16 glyph and the text at 16pt (so iOS doesn't
	// zoom), 8 apart. Search is the desktop's 30px box (its props are the old app's twin's).
	type Props = { placeholder: string; value?: string; label?: string; oninput?: (value: string) => void };
	let { placeholder, value = $bindable(''), label, oninput }: Props = $props();
</script>

<label class="flex h-9.5 w-full items-center gap-2 rounded-lg border border-transparent bg-surface-2 px-2.75 text-text-3 focus-within:border-accent focus-within:shadow-focus">
	<Icon icon={SearchIcon} size={16} />
	<input
		type="search"
		{placeholder}
		aria-label={label ?? placeholder}
		bind:value
		oninput={() => oninput?.(value)}
		class="min-w-0 flex-1 bg-transparent text-phone text-text outline-none placeholder:text-text-3"
	/>
</label>
