<script lang="ts" module>
	import type { Component, Snippet } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type SettingsRow = {
		/** Where the row goes (a page); without it, `onclick` makes it a button (a sheet, a screen). */
		href?: string;
		onclick?: () => void;
		icon?: Component<LucideProps>;
		/** Leads the row in place of the icon (a crew member's mascot). */
		lead?: Snippet;
		label: string;
		/** A second line under the label. */
		sub?: string;
		/** What's set, at the row's end. */
		meta?: string;
		/** The value is machine-written (an address, a zone, a model): mono. */
		mono?: boolean;
		/** A value that means something: the accent (connected, working) or the warning (waiting). */
		tone?: 'accent' | 'warning';
		/** A switch at the row's end, saved as it flips. */
		toggle?: { checked: boolean; onchange: (checked: boolean) => void; disabled?: boolean };
		/** The chosen one of a sheet's choices. */
		chosen?: boolean;
		/** Destructive: the danger colour, and no chevron. */
		danger?: boolean;
		/** Busy with what it started (a disconnect, a sign-out). */
		busy?: boolean;
		/** Keeps rows apart when two share a label or a link. */
		key?: string;
	};
</script>

<script lang="ts">
	import { Check, ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Toggle from '../inputs/Toggle.svelte';

	// Flow / Mobile — You & Settings (EtSi2): a group of settings rows. Its label in mono capitals
	// (with a note at the end where it has one), then the rows in a card (surface, border, radius 14):
	// 52 high, or 12 above and below with a second line; the icon at 18 (or a mascot), the label at
	// 16 and its line at 13, what's set at 15 (mono at 11 for machine values), then a chevron, a
	// switch, or the tick of a chosen choice. A row goes somewhere (a link), opens something (a
	// button), or holds its switch; a switch row's label names the switch.
	type Props = { label?: string; note?: string; rows: SettingsRow[] };
	let { label, note, rows }: Props = $props();
	const id = $props.id();
</script>

<section aria-labelledby={label ? id : undefined} aria-label={label ? undefined : rows[0]?.label} class="flex flex-col gap-2">
	{#if label}
		<div class="flex items-center gap-2 px-1">
			<h2 {id} class="min-w-0 flex-1 font-mono text-label tracking-label text-text-3 uppercase">{label}</h2>
			{#if note}<span class="shrink-0 font-mono text-label text-text-3">{note}</span>{/if}
		</div>
	{/if}
	<ul class="flex flex-col rounded-xl bg-surface outline -outline-offset-1 outline-border">
		{#each rows as row, index (row.key ?? (row.href ?? '') + row.label)}
			{@const last = index === rows.length - 1}
			<li class={['flex', !last && 'border-b border-border']}>
				{#snippet body()}
					{#if row.lead}
						<span class="inline-flex shrink-0">{@render row.lead()}</span>
					{:else if row.icon}
						<span class={['inline-flex shrink-0', row.danger ? 'text-danger' : 'text-text-2']}><Icon icon={row.icon} size={18} /></span>
					{/if}
					<span class="flex min-w-0 flex-1 flex-col gap-0.5 text-left">
						<span class={['truncate text-phone', row.danger ? 'text-danger' : 'text-text']}>{row.label}</span>
						{#if row.sub}<span class="text-phone-sm text-text-3">{row.sub}</span>{/if}
					</span>
					{#if row.meta}
						<span class={['max-w-1/2 shrink-0 truncate', row.mono ? 'font-mono text-label' : 'text-phone-sm', row.tone === 'accent' ? 'text-accent' : row.tone === 'warning' ? 'text-warning' : 'text-text-3']}>{row.meta}</span>
					{/if}
					{#if row.chosen}
						<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={18} /></span>
					{:else if !row.danger && !row.toggle && (row.href || row.onclick)}
						<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={14} /></span>
					{/if}
				{/snippet}
				{#if row.toggle}
					<div class={['flex w-full items-center gap-3 px-3.5', row.sub ? 'py-3' : 'h-13']}>
						{@render body()}
						<span class="inline-flex shrink-0 pointer-coarse:p-2"><Toggle label={row.label} checked={row.toggle.checked} disabled={row.toggle.disabled} onchange={row.toggle.onchange} /></span>
					</div>
				{:else if row.href}
					<a href={row.href} class={['flex w-full items-center gap-3 px-3.5 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus', row.sub ? 'py-3' : 'h-13']}>{@render body()}</a>
				{:else if row.onclick}
					<button type="button" aria-pressed={row.chosen === undefined ? undefined : row.chosen} disabled={row.busy} class={['flex w-full cursor-pointer items-center gap-3 px-3.5 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus disabled:cursor-default disabled:opacity-60', row.sub ? 'py-3' : 'h-13']} onclick={row.onclick}>{@render body()}</button>
				{:else}
					<div class={['flex w-full items-center gap-3 px-3.5', row.sub ? 'py-3' : 'h-13']}>{@render body()}</div>
				{/if}
			</li>
		{/each}
	</ul>
</section>
