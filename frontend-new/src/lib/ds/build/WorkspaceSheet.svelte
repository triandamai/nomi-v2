<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { Maximize2, X, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import IconButton from '../actions/IconButton.svelte';

	// Build → Workspace / Sheet: every project tool (domain, database, settings…) opens as a sheet
	// over the preview, so the app you're looking at never closes. Escape closes it.
	type Props = {
		icon: Component<LucideProps>;
		title: string;
		/** The project, in mono. */
		meta?: string;
		onexpand?: () => void;
		onclose: () => void;
		children: Snippet;
	};
	let { icon, title, meta, onexpand, onclose, children }: Props = $props();

	const id = $props.id();
</script>

<!-- Escape closes the sheet from anywhere inside it. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section
	aria-labelledby="{id}-title"
	tabindex="-1"
	class="flex h-full w-full flex-col overflow-hidden border-l border-border-strong bg-surface outline-none"
	onkeydown={(event) => {
		if (event.key === 'Escape') {
			event.preventDefault();
			onclose();
		}
	}}
>
	<header class="flex items-center gap-2.5 border-b border-border pt-2.5 pr-2.5 pb-2.25 pl-3.75">
		<span class="inline-flex text-text-2"><Icon {icon} size={14} /></span>
		<h2 id="{id}-title" class="text-ui font-medium text-text">{title}</h2>
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{meta}</span>
		{#if onexpand}<IconButton icon={Maximize2} label={m.ws_expand()} variant="ghost" onclick={onexpand} />{/if}
		<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={onclose} />
	</header>
	<div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5">{@render children()}</div>
	<footer class="border-t border-border pt-2.25 pr-4 pb-2.5 pl-3.75 font-mono text-label text-text-3">{m.ws_sheet_hint()}</footer>
</section>
