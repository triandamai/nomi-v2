<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type AdminNavItem = { href: string; icon: Component<LucideProps>; label: string; count?: number | string };
	export type AdminNavGroup = { label: string; items: AdminNavItem[] };
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ArrowLeft } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import NavItem from '../navigation/NavItem.svelte';
	import { setRowStyle } from '../navigation/rowStyle';

	// Admin → Admin nav: the admin console's sidebar, built like Agents & app chrome → Settings nav
	// (248 wide, rows 10px in with text-3 icons, groups under mono capitals): the brand with an
	// "admin" mark so it's never taken for the app, the way back to Nomi, the areas the person may
	// open (only those), and the person at the foot (`footer`: the account menu). The groups scroll
	// on a short screen; the brand and the foot stay.
	type Props = {
		/** Names the navigation ("Admin"). */
		label: string;
		/** The mark beside the wordmark ("admin"). */
		mark: string;
		homeHref: string;
		backHref: string;
		backLabel: string;
		groups: AdminNavGroup[];
		active?: string;
		footer?: Snippet;
	};
	let { label, mark, homeHref, backHref, backLabel, groups, active, footer }: Props = $props();

	setRowStyle({ dense: true, icon: 'text-3' });
</script>

<nav aria-label={label} class="flex h-full w-62 shrink-0 flex-col gap-3.5 border-r border-border bg-bg py-4 pr-2.75 pl-3">
	<a href={homeHref} class="flex items-center gap-2 rounded-sm px-2 py-1 outline-offset-1">
		<Mascot agent="nomi" size={20} />
		<span class="text-body font-semibold tracking-wordmark text-text">nomi</span>
		<span class="rounded-xs border border-border-strong px-1.25 py-px font-mono text-label tracking-label text-text-2 uppercase">{mark}</span>
	</a>
	<a href={backHref} class="flex h-8 items-center gap-2.5 rounded-sm px-2.5 text-ui text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:min-h-11">
		<span class="inline-flex text-text-3"><Icon icon={ArrowLeft} /></span>
		<span class="flex-1">{backLabel}</span>
	</a>
	<div class="flex min-h-0 flex-1 flex-col gap-3.5 overflow-y-auto">
		{#each groups as group (group.label)}
			<div class="flex flex-col gap-0.5">
				<p class="px-2.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{group.label}</p>
				{#each group.items as item (item.href)}
					<NavItem href={item.href} icon={item.icon} label={item.label} count={item.count} active={item.href === active} />
				{/each}
			</div>
		{/each}
	</div>
	{#if footer}<div class="flex flex-col gap-1 border-t border-border pt-1.75">{@render footer()}</div>{/if}
</nav>
