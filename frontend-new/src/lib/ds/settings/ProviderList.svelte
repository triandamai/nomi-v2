<script lang="ts">
	import { ChevronRight, Plus, Search as SearchIcon } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import ProviderTile from './ProviderTile.svelte';

	// App / Settings / Models & providers / Connect (voUxB) → Provider List: the search on top, then
	// the providers in groups (Connected, Popular, then A to Z) as 30px rows, the one being connected
	// on surface-2, a ✓ in the accent on those already connected; the letters down the side jump to
	// their group; any OpenAI-compatible address at the foot, on the surface.
	export type ListProvider = { id: string; name: string; connected?: boolean };
	type Props = {
		search: string;
		query?: string;
		sections: { label: string; providers: ListProvider[] }[];
		chosen?: string;
		empty: string;
		custom: { title: string; detail: string };
		onpick?: (id: string) => void;
	};
	let { search, query = $bindable(''), sections, chosen, empty, custom, onpick }: Props = $props();

	const id = $props.id();
	const letters = $derived(sections.filter((section) => section.label.length === 1));
	let list: HTMLElement | undefined = $state();
	const jump = (label: string) => list?.querySelector(`[data-section="${CSS.escape(label)}"]`)?.scrollIntoView({ block: 'start' });
</script>

<div class="flex flex-col gap-2 border-b border-border px-3 pt-3 pb-1.75">
	<label class="flex h-8.5 items-center gap-2 rounded-sm border border-border-strong bg-surface px-2.25 focus-within:border-accent focus-within:shadow-focus">
		<span class="inline-flex text-text-3"><Icon icon={SearchIcon} size={14} /></span>
		<input bind:value={query} placeholder={search} aria-label={search} class="min-w-0 flex-1 bg-transparent text-ui text-text caret-accent outline-none placeholder:text-text-3" />
	</label>
</div>
<div class="flex min-h-0 flex-1">
	<div bind:this={list} class="min-h-0 flex-1 overflow-y-auto py-1.5 pr-1 pl-2">
		{#each sections as section (section.label)}
			<p data-section={section.label} class="flex scroll-mt-1 gap-1.5 px-2 pt-2 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{section.label}</p>
			<ul role="list" aria-label={section.label} class="flex flex-col gap-px">
				{#each section.providers as provider (provider.id)}
					<li>
						<button
							type="button"
							aria-current={provider.id === chosen ? 'true' : undefined}
							onclick={() => onpick?.(provider.id)}
							class={['flex h-7.5 w-full cursor-pointer items-center gap-2.5 rounded-sm px-2 text-left text-ui outline-offset-1 hover:bg-surface-2', provider.id === chosen ? 'bg-surface-2 font-medium text-text' : 'text-text']}
						>
							<ProviderTile name={provider.name} size={20} />
							<span class="min-w-0 flex-1 truncate">{provider.name}</span>
							{#if provider.connected}<span class="font-mono text-label text-accent" aria-label={m.models_connected()}>✓</span>{/if}
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<p class="px-2 py-3 text-caption text-text-3">{empty}</p>
		{/each}
	</div>
	{#if letters.length > 1}
		<nav aria-label={m.models_letters()} class="flex w-4.5 shrink-0 flex-col items-center justify-between py-2">
			{#each letters as section (section.label)}
				<button type="button" aria-label={section.label} onclick={() => jump(section.label)} class="cursor-pointer font-mono text-nano text-text-3 hover:text-text">{section.label}</button>
			{/each}
		</nav>
	{/if}
</div>
<button type="button" id="{id}-custom" onclick={() => onpick?.('custom')} class="flex shrink-0 cursor-pointer items-center gap-2.5 border-t border-border bg-surface px-3 pt-2.25 pb-2.5 text-left outline-offset-1 hover:bg-surface-2">
	<span class="inline-flex size-7 shrink-0 items-center justify-center rounded-sm text-text outline -outline-offset-1 outline-border-strong"><Icon icon={Plus} size={14} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-px">
		<span class="text-ui font-medium text-text">{custom.title}</span>
		<span class="truncate text-caption text-text-3">{custom.detail}</span>
	</span>
	<span class="inline-flex text-text-3"><Icon icon={ChevronRight} size={14} /></span>
</button>
