<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { rowStyle } from './rowStyle';

	// Design System / Nomi → Navigation: a sidebar row. Active = surface-2 fill + primary text. The
	// container decides how far in it sits and its icon's tone (rowStyle).
	type Props = { href: string; icon: Component<LucideProps>; label: string; count?: number | string; active?: boolean };
	let { href, icon, label, count, active = false }: Props = $props();

	const style = rowStyle();
</script>

<a
	{href}
	aria-current={active ? 'page' : undefined}
	class={[
		'flex h-8 w-full items-center rounded-sm text-ui outline-offset-1 pointer-coarse:min-h-11',
		style.dense ? 'gap-2.5 px-2.5' : 'gap-3 px-3',
		active ? 'bg-surface-2 font-medium text-text' : 'text-text-2 hover:bg-surface-2 hover:text-text',
	]}
>
	<span class={['inline-flex', active && 'text-text', !active && style.icon === 'text-2' && 'text-text-2', !active && style.icon === 'text-3' && 'text-text-3']}><Icon {icon} /></span>
	<span class="min-w-0 flex-1 truncate">{label}</span>
	{#if count !== undefined}<span class="font-mono text-label font-normal text-text-3">{count}</span>{/if}
</a>
