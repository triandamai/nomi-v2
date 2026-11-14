<script lang="ts">
	import { Code, ExternalLink, Eye, Monitor, RotateCw, Smartphone, SquareTerminal, X } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Build → Screen / Building (H4DIH) → Pane Tabs: the preview pane's strip, 44 high,
	// [0,12,0,16] and 12 apart over its line. The view switch (Preview, Code, Terminal: 26 high
	// segments on surface-2, the chosen one on the page with a border-strong edge), the URL bar (30,
	// outlined: "live" with its dot when it's the live app, the address, the deploy, Reload, Open),
	// the device switch (desktop or phone, the chosen on surface-2) and Close.
	export type PaneView = 'preview' | 'code' | 'terminal';
	type Props = {
		labels: { views: string; preview: string; code: string; terminal: string; device: string; desktop: string; phone: string; reload: string; open: string; close: string };
		view?: PaneView;
		/** The views on offer, in order (all three by default). */
		views?: PaneView[];
		device?: 'desktop' | 'phone';
		/** What the preview shows: the live app ("live" set) or the project's own files. */
		address?: { url: string; shown: string; live?: string; deploy?: string };
		onreload?: () => void;
		onclose?: () => void;
	};
	let { labels, view = $bindable('preview'), views: offered = ['preview', 'code', 'terminal'], device = $bindable('desktop'), address, onreload, onclose }: Props = $props();

	const views = $derived(
		[
			{ value: 'preview' as const, label: labels.preview, icon: Eye },
			{ value: 'code' as const, label: labels.code, icon: Code },
			{ value: 'terminal' as const, label: labels.terminal, icon: SquareTerminal },
		].filter((option) => offered.includes(option.value)),
	);
	const devices = $derived([
		{ value: 'desktop' as const, label: labels.desktop, icon: Monitor },
		{ value: 'phone' as const, label: labels.phone, icon: Smartphone },
	]);
</script>

<div data-part="tabs" class="flex h-11 shrink-0 items-center gap-3 border-b border-border pr-3 pl-4">
	<div role="radiogroup" aria-label={labels.views} class="flex shrink-0 gap-0.5 rounded-sm bg-surface-2 p-0.5">
		{#each views as option (option.value)}
			{@const chosen = option.value === view}
			<button
				type="button"
				role="radio"
				aria-checked={chosen}
				onclick={() => (view = option.value)}
				class={[
					'inline-flex h-6.5 cursor-pointer items-center gap-1.5 rounded-sm border px-2.25 text-ui outline-offset-1',
					chosen ? 'border-border-strong bg-bg font-medium text-text' : 'border-transparent text-text-2 hover:text-text',
				]}
			>
				<Icon icon={option.icon} size={14} /><span class="max-sm:sr-only">{option.label}</span>
			</button>
		{/each}
	</div>
	{#if address}
		<div class="flex h-7.5 min-w-0 flex-1 items-center gap-2 rounded-sm px-2.5 font-mono text-label outline -outline-offset-1 outline-border">
			{#if address.live}
				<span class="inline-flex shrink-0 items-center gap-1.25 text-text-2"><span aria-hidden="true" class="size-1.5 rounded-full bg-accent"></span>{address.live}</span>
				<span aria-hidden="true" class="h-3.5 w-px shrink-0 bg-border"></span>
			{/if}
			<span class="min-w-0 flex-1 truncate text-text-2">{address.shown}</span>
			{#if address.deploy}<span class="shrink-0 text-text-3">{address.deploy}</span>{/if}
			<button type="button" aria-label={labels.reload} onclick={onreload} class="inline-flex size-5 shrink-0 cursor-pointer items-center justify-center rounded-xs text-text-2 outline-offset-1 hover:text-text">
				<Icon icon={RotateCw} size={14} />
			</button>
			<a href={address.url} target="_blank" rel="noopener" aria-label={labels.open} class="inline-flex size-5 shrink-0 items-center justify-center rounded-xs text-text-2 outline-offset-1 hover:text-text">
				<Icon icon={ExternalLink} size={14} />
			</a>
		</div>
	{:else}
		<span class="flex-1"></span>
	{/if}
	<div role="radiogroup" aria-label={labels.device} class="flex shrink-0 gap-0.5 max-sm:hidden">
		{#each devices as option (option.value)}
			<button
				type="button"
				role="radio"
				aria-checked={option.value === device}
				aria-label={option.label}
				onclick={() => (device = option.value)}
				class={['inline-flex size-7.5 cursor-pointer items-center justify-center rounded-sm text-text-2 outline-offset-1 hover:text-text', option.value === device && 'bg-surface-2']}
			>
				<Icon icon={option.icon} />
			</button>
		{/each}
	</div>
	{#if onclose}
		<button type="button" aria-label={labels.close} onclick={onclose} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm text-text-2 outline-offset-1 hover:bg-surface-2 hover:text-text">
			<Icon icon={X} />
		</button>
	{/if}
</div>
