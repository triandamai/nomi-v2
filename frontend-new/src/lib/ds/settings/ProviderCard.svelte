<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronDown, ChevronRight, Star } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import ProviderTile from './ProviderTile.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Models & providers (RbTgo) → Provider: one provider's models, a card that
	// opens (border-strong and its header on the surface when open). The header: the chevron, the
	// tile, the name with who pays, what it holds in the picker in mono, then its tools. Open, a
	// table of its models (name in mono, "★ Default for chats", this month's tokens, In picker),
	// 58 in from the left to sit under the name.
	export type CardModel = { key: string; label: string; enabled: boolean; isDefault: boolean; month: string };
	type Props = {
		name: string;
		/** Nomi's own models wear Nomi's mascot in place of an initial. */
		mascot?: CrewId;
		detail?: string;
		tag?: Snippet;
		count: string;
		models: CardModel[];
		columns: { model: string; month: string; picker: string };
		defaultLabel: string;
		empty: string;
		open?: boolean;
		status?: Snippet;
		tools?: Snippet;
		onshow?: (key: string, shown: boolean) => void;
	};
	let { name, mascot, detail, tag, count, models, columns, defaultLabel, empty, open = $bindable(false), status, tools, onshow }: Props = $props();

	const id = $props.id();
</script>

<section aria-labelledby="{id}-name" class={['overflow-hidden rounded-md outline -outline-offset-1', open ? 'outline-border-strong' : 'outline-border']}>
	<header class={['flex items-center gap-3 px-3.5 py-3', open && 'border-b border-border bg-surface pb-2.75']}>
		<button
			type="button"
			aria-expanded={open}
			aria-controls="{id}-models"
			onclick={() => (open = !open)}
			class="flex min-w-0 flex-1 cursor-pointer items-center gap-3 rounded-sm text-left outline-offset-2"
		>
			<span class="inline-flex text-text-2"><Icon icon={open ? ChevronDown : ChevronRight} /></span>
			<ProviderTile {name} {mascot} />
			<span class="flex min-w-0 flex-1 flex-col gap-0.75">
				<span class="flex min-h-5 min-w-0 items-center gap-2">
					<span id="{id}-name" class="truncate text-ui font-medium text-text">{name}</span>
					{@render tag?.()}
				</span>
				{#if detail}<span class="truncate font-mono text-label text-text-3">{detail}</span>{/if}
			</span>
		</button>
		{@render status?.()}
		<span class="shrink-0 font-mono text-label text-text-2">{count}</span>
		{@render tools?.()}
	</header>
	{#if open}
		<div id="{id}-models">
			{#if models.length}
				<table class="w-full">
					<thead>
						<tr class="border-b border-border font-mono text-label tracking-label whitespace-nowrap text-text-3 uppercase">
							<th class="py-2 pl-14.5 text-left font-normal">{columns.model}</th>
							<th class="w-21 py-2 text-left font-normal">{columns.month}</th>
							<th class="w-18 py-2 pr-3.5 text-right font-normal">{columns.picker}</th>
						</tr>
					</thead>
					<tbody>
						{#each models as model (model.key)}
							<tr class="group border-b border-border last:border-b-0">
								<td class="pt-2.25 pb-2 pl-14.5 group-last:pb-2.25">
									<span class="flex flex-wrap items-center gap-2">
										<span class="font-mono text-ui text-text">{model.label}</span>
										{#if model.isDefault}
											<span class="inline-flex h-5 items-center gap-1 rounded-xs px-1.5 font-mono text-label text-text-2 outline -outline-offset-1 outline-border-strong">
												<Icon icon={Star} size={12} />{defaultLabel}
											</span>
										{/if}
									</span>
								</td>
								<td class={['pt-2.25 pb-2 font-mono text-label group-last:pb-2.25', model.month === '—' ? 'text-text-3' : 'text-text-2']}>{model.month}</td>
								<td class="pt-2.25 pb-2 pr-3.5 text-right group-last:pb-2.25">
									<Toggle label={model.label} checked={model.enabled} onchange={(shown) => onshow?.(model.key, shown)} />
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{:else}
				<p class="py-3 pl-14.5 text-caption text-text-3">{empty}</p>
			{/if}
		</div>
	{/if}
</section>
