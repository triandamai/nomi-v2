<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	export type Tab = { href: string; label: string; icon: Component<LucideProps>; current?: boolean };
</script>

<script lang="ts">
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Start & answer → Chats → Tab Bar: a phone's way around the app in place of
	// the sidebar ("Chats is a tab, not a drawer"). 50 tall on the ground over a top stroke, [8,12,0,12],
	// five equal tabs, each its 22 icon over its name at 10, 3 apart: text-3, the current one text
	// and 600. Room for the home indicator under it once the app is on the home screen (pb-safe).
	type Props = { label: string; tabs: Tab[] };
	let { label, tabs }: Props = $props();
</script>

<nav aria-label={label} class="flex shrink-0 border-t border-border bg-bg px-3 pt-1.75 pb-safe">
	{#each tabs as tab (tab.href)}
		<a
			href={tab.href}
			aria-current={tab.current ? 'page' : undefined}
			class={['flex h-10.5 min-w-0 flex-1 flex-col items-center gap-0.75 py-0.5 -outline-offset-2', tab.current ? 'text-text' : 'text-text-3']}
		>
			<Icon icon={tab.icon} size={22} />
			<span class={['truncate text-micro', tab.current && 'font-semibold']}>{tab.label}</span>
		</a>
	{/each}
</nav>
