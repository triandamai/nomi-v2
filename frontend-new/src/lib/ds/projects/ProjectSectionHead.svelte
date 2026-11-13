<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Link from '../actions/Link.svelte';

	// Design System / Projects → Project / Section Head (TYcRO): a row of the projects page that
	// isn't the user's own projects ("Remix from the community"). Its 15px icon and title (15,
	// semibold: the pen's weight) over one line in text-3, 4 apart; a quiet link at the bottom of
	// the other end. It spans the grid.
	type Props = {
		icon: Component<LucideProps>;
		title: string;
		description: string;
		link?: { label: string; href: string; external?: boolean };
	};
	let { icon, title, description, link }: Props = $props();
</script>

<header class="flex w-full items-end justify-between gap-4">
	<div class="flex min-w-0 flex-col gap-1">
		<h2 class="flex items-center gap-2 text-body font-semibold text-text"><Icon {icon} size={15} />{title}</h2>
		<p class="text-ui text-text-3">{description}</p>
	</div>
	{#if link}<span class="inline-flex shrink-0"><Link href={link.href} label={link.label} external={link.external} quiet /></span>{/if}
</header>
