<script lang="ts">
	import type { Snippet } from 'svelte';
	import { PanelLeft, Plus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import { setRowStyle } from '../navigation/rowStyle';

	// Agents & app chrome → Sidebar: every app-shell screen's navigation. Brand and collapse, New
	// chat (⌘K), the sections (children: NavItems and group labels), recents, then usage and the
	// person at the foot.
	type Props = {
		newChatHref: string;
		newChatShortcut?: string;
		oncollapse?: () => void;
		children: Snippet;
		recents?: Snippet;
		footer?: Snippet;
	};
	let { newChatHref, newChatShortcut = '⌘K', oncollapse, children, recents, footer }: Props = $props();

	setRowStyle({ dense: true, icon: 'text-2' });
</script>

<nav aria-label={m.nav_main()} class="flex h-full w-62 shrink-0 flex-col gap-5 border-r border-border bg-bg px-3 py-4">
	<div class="flex items-center justify-between px-2 py-1">
		<span class="inline-flex items-center gap-2">
			<Mascot agent="nomi" size={20} />
			<span class="text-body font-semibold tracking-wordmark text-text">nomi</span>
		</span>
		{#if oncollapse}
			<button type="button" aria-label={m.nav_collapse()} class="inline-flex cursor-pointer items-center justify-center rounded-xs text-text-2 outline-offset-2 hover:text-text pointer-coarse:size-11" onclick={oncollapse}>
				<Icon icon={PanelLeft} />
			</button>
		{/if}
	</div>
	<a href={newChatHref} class="flex h-8.5 w-full items-center gap-2 rounded-sm border border-border-strong bg-surface px-2.5 text-ui font-medium text-text outline-offset-1 hover:bg-surface-2 pointer-coarse:min-h-11">
		<Icon icon={Plus} />
		<span class="flex-1">{m.nav_new_chat()}</span>
		<span class="font-mono text-label font-normal text-text-3">{newChatShortcut}</span>
	</a>
	<div class="flex flex-col gap-0.5">{@render children()}</div>
	{#if recents}<div class="flex min-h-0 flex-1 flex-col overflow-y-auto">{@render recents()}</div>{:else}<div class="flex-1"></div>{/if}
	{#if footer}<div class="flex flex-col gap-3">{@render footer()}</div>{/if}
</nav>
