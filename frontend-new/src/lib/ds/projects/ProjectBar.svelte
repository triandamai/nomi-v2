<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { ArrowUpRight, ChevronLeft, Download, Share2, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Build → Screen / Building (H4DIH) → Project Bar: a project's own bar, 52 high, 16 in and
	// 14 apart, its line drawn along the bottom. Back (28), the crew member who builds it (20), the
	// name (13/500) with its status (an outlined pill: its dot and the word in mono), and under it
	// where it lives (the workspace in mono, then the address, linked); then the tools (30 high,
	// [0,10], icon and label, the open one on surface-2), a divider, Share, Export, `publish` and the
	// person's initials. On a phone the tools and the address step aside. Tools and Share show only
	// once there's something behind them.
	export type ProjectTool = { id: string; label: string; icon: Component<LucideProps> };
	type Props = {
		backHref: string;
		backLabel: string;
		agent: CrewId;
		name: string;
		status: { label: string; tone: 'accent' | 'warning' | 'danger' | 'quiet' };
		/** The workspace it's in. */
		space: string;
		/** The live address. */
		url?: string;
		nav: ProjectTool[];
		navLabel: string;
		active?: string;
		shareLabel: string;
		exportLabel: string;
		initials: string;
		publish?: Snippet;
		/**
		 * Who's here (Invite & build together TnwQT: Presence), in place of the person's own face
		 * while someone else has the project open.
		 */
		presence?: Snippet;
		onnav?: (id: string) => void;
		onshare?: () => void;
		onexport?: () => void;
	};
	let { backHref, backLabel, agent, name, status, space, url, nav, navLabel, active, shareLabel, exportLabel, initials, publish, presence, onnav, onshare, onexport }: Props = $props();

	const shown = $derived(url?.replace(/^https?:\/\//, '').replace(/\/$/, ''));
</script>

<header class="flex h-13 shrink-0 items-center gap-3.5 border-b border-border px-4">
	<a href={backHref} aria-label={backLabel} class="inline-flex size-7 shrink-0 items-center justify-center rounded-sm text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text">
		<Icon icon={ChevronLeft} />
	</a>
	<Mascot {agent} size={20} />
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<div class="flex min-w-0 items-center gap-2.5">
			<span class="truncate text-ui font-medium text-text">{name}</span>
			<span class="inline-flex shrink-0 items-center gap-1.5 rounded-sm px-1.5 py-0.5 outline -outline-offset-1 outline-border-strong">
				<span
					aria-hidden="true"
					class={[
						'size-1.5 rounded-full',
						status.tone === 'accent' && 'bg-accent',
						status.tone === 'warning' && 'bg-warning',
						status.tone === 'danger' && 'bg-danger',
						status.tone === 'quiet' && 'bg-text-3',
					]}
				></span>
				<span class="font-mono text-label text-text-2">{status.label}</span>
			</span>
		</div>
		<div class="flex min-w-0 items-center gap-1.5 font-mono text-label max-sm:hidden">
			<span class="shrink-0 text-text-3">{space}</span>
			{#if url}
				<span class="text-text-3">/</span>
				<a href={url} target="_blank" rel="noopener" class="inline-flex min-w-0 items-center gap-1.5 rounded-xs text-text-2 outline-offset-1 hover:text-text">
					<span class="truncate">{shown}</span><span class="inline-flex shrink-0"><Icon icon={ArrowUpRight} size={12} /></span>
				</a>
			{/if}
		</div>
	</div>
	{#if nav.length}
	<nav aria-label={navLabel} class="flex shrink-0 items-center gap-0.5 max-lg:hidden">
		{#each nav as tool (tool.id)}
			<button
				type="button"
				aria-pressed={active === tool.id}
				onclick={() => onnav?.(tool.id)}
				class={['inline-flex h-7.5 cursor-pointer items-center gap-1.5 rounded-sm px-2.5 text-ui outline-offset-1', active === tool.id ? 'bg-surface-2 text-text' : 'text-text-2 hover:bg-surface-2 hover:text-text']}
			>
				<Icon icon={tool.icon} size={14} />{tool.label}
			</button>
		{/each}
	</nav>
	<span aria-hidden="true" class="h-5 w-px shrink-0 bg-border max-lg:hidden"></span>
	{/if}
	{#if onshare}
	<button type="button" onclick={onshare} class="inline-flex h-7.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong px-2.75 text-ui text-text outline-offset-1 hover:bg-surface-2">
		<Icon icon={Share2} size={14} />{shareLabel}
	</button>
	{/if}
	<button type="button" aria-label={exportLabel} title={exportLabel} onclick={onexport} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm border border-border-strong text-text outline-offset-1 hover:bg-surface-2">
		<Icon icon={Download} size={14} />
	</button>
	{@render publish?.()}
	{#if presence}
		{@render presence()}
	{:else}
		<span aria-hidden="true" class="inline-flex size-7 shrink-0 items-center justify-center rounded-sm bg-surface-2 font-mono text-label text-text-2 outline -outline-offset-1 outline-border-strong max-sm:hidden">{initials}</span>
	{/if}
</header>
