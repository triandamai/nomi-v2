<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Link from '../actions/Link.svelte';

	// Nothing here yet, said plainly, with the one thing to do about it.
	type Props = { icon: Component<LucideProps>; title: string; count?: number; body: string; action?: { label: string; href: string } };
	let { icon, title, count, body, action }: Props = $props();
</script>

<section class="flex w-full flex-col gap-2 rounded-md border border-border p-4">
	<header class="flex items-center gap-2">
		<span class="inline-flex text-text-3"><Icon {icon} size={14} /></span>
		<span class="min-w-0 flex-1 text-ui font-medium text-text">{title}</span>
		{#if count !== undefined}<span class="font-mono text-label text-text-3">{count}</span>{/if}
	</header>
	<p class="text-ui leading-normal text-text-2">{body}</p>
	{#if action}<Link href={action.href} label={action.label} />{/if}
</section>
