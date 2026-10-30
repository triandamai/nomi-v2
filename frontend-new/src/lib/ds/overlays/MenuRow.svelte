<script lang="ts">
	import Icon from '../foundations/Icon.svelte';
	import type { MenuItem } from './MenuPanel.svelte';

	// Design System / Nomi → Overlays / Menu Item: 30 tall, icon, label, shortcut or meta; danger in
	// red. `hover` shows the hover without a pointer (the gallery). Lives in a role="menu".
	// `wide` rows (the chat's regenerate and attach menus) are 32 tall with 10 between their parts.
	type Props = { item: MenuItem; wide?: boolean; onpick?: (item: MenuItem) => void };
	let { item, wide = false, onpick }: Props = $props();
</script>

<button
	type="button"
	role="menuitem"
	tabindex="-1"
	data-hover={item.hover || undefined}
	onclick={() => onpick?.(item)}
	class={[
		'flex w-full cursor-pointer items-center rounded-xs px-2 text-left text-ui outline-none hover:bg-surface-2 focus-visible:bg-surface-2 data-hover:bg-surface-2 pointer-coarse:min-h-11',
		wide ? 'h-8 gap-2.5' : 'h-7.5 gap-2',
		item.danger ? 'text-danger' : 'text-text',
	]}
>
	{#if item.icon}<span class={['inline-flex', item.danger ? 'text-danger' : 'text-text-2']}><Icon icon={item.icon} size={14} /></span>{/if}
	<span class="min-w-0 flex-1 truncate">{item.label}</span>
	{#if item.meta}<span class="font-mono text-label text-text-3">{item.meta}</span>{/if}
	{#if item.shortcut}<span class="font-mono text-label text-text-3">{item.shortcut}</span>{/if}
</button>
