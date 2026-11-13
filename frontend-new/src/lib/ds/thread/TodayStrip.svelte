<script lang="ts" module>
	export type TodayColumnProps = {
		key: 'out' | 'reminders' | 'plans';
		title: string;
		count: number;
		body: string;
		action: { label: string; href?: string; onclick?: () => void };
	};
</script>

<script lang="ts">
	import { ArrowRight, Bell, Inbox, ListChecks } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// App / Home → Today Strip (BgqD6): "TODAY" and when it was last updated, over one bordered
	// panel of three columns (each padded 16, 10 apart, the second and third with a stroke on the
	// left): what came in while you were out, today's reminders, plans running. Each heads with its
	// 26px tile (in Nomi's or Rena's tint, as drawn), its name and a count, then a line and a link.
	// A phone stacks the columns.
	type Props = { columns: TodayColumnProps[]; updated?: string };
	let { columns, updated }: Props = $props();

	const id = $props.id();
	const ICONS = { out: Inbox, reminders: Bell, plans: ListChecks };
</script>

<section aria-labelledby="{id}-title" class="flex w-full flex-col gap-2.5">
	<header class="flex items-center justify-between font-mono text-label text-text-3">
		<h2 id="{id}-title" class="tracking-label uppercase">{m.today_title()}</h2>
		{#if updated}<span>{m.today_updated({ time: updated })}</span>{/if}
	</header>
	<div class="grid w-full rounded-md outline -outline-offset-1 outline-border md:grid-cols-3">
		{#each columns as column, index (column.key)}
			<article class={['flex min-w-0 flex-col gap-2.5 p-4', index > 0 && 'max-md:border-t max-md:border-border max-md:pt-3.75 md:border-l md:border-border md:pl-3.75']}>
				<h3 class="flex items-center gap-2">
					<span class={['inline-flex size-6.5 shrink-0 items-center justify-center rounded-tile text-text-2', column.key === 'out' ? 'bg-crew-nomi-bg' : 'bg-crew-rena-bg']}>
						<Icon icon={ICONS[column.key]} size={14} />
					</span>
					<span class="min-w-0 flex-1 truncate text-ui font-medium text-text">{column.title}</span>
					<span class="font-mono text-label text-text-3">{column.count}</span>
				</h3>
				<p class="text-ui leading-normal text-text-2">{column.body}</p>
				{#if column.action.href}
					<a href={column.action.href} class="inline-flex items-center gap-1.5 self-start rounded-xs text-ui text-text outline-offset-2 hover:underline hover:decoration-border-strong hover:underline-offset-2 pointer-coarse:min-h-11">
						{column.action.label}<span class="inline-flex text-text-3"><Icon icon={ArrowRight} size={14} /></span>
					</a>
				{:else}
					<button type="button" onclick={column.action.onclick} class="inline-flex cursor-pointer items-center gap-1.5 self-start rounded-xs text-ui text-text outline-offset-2 hover:underline hover:decoration-border-strong hover:underline-offset-2 pointer-coarse:min-h-11">
						{column.action.label}<span class="inline-flex text-text-3"><Icon icon={ArrowRight} size={14} /></span>
					</button>
				{/if}
			</article>
		{/each}
	</div>
</section>
