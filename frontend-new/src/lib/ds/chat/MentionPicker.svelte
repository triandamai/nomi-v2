<script lang="ts">
	import { File } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { MentionOption } from './mention';

	// Chat responses → Mention Picker: opens under "@" in the composer, the crew first, then files.
	// Focus stays in the composer: it moves `active` (nextOption) and points at `<id>-option-<n>`
	// with aria-activedescendant; a click picks without taking focus away. `active` counts in the
	// order shown, crew then files.
	type Props = { id: string; options: MentionOption[]; active?: number; onselect: (option: MentionOption) => void };
	let { id, options, active = $bindable(0), onselect }: Props = $props();

	const crew = $derived(options.filter((option) => option.kind === 'crew'));
	const files = $derived(options.filter((option) => option.kind === 'file'));
</script>

{#snippet item(option: MentionOption, index: number)}
	<!-- Keys are handled by the composer, which keeps focus (aria-activedescendant). -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		id="{id}-option-{index}"
		role="option"
		tabindex="-1"
		aria-selected={index === active}
		class={['flex h-8.5 cursor-pointer items-center gap-2 rounded-xs px-2 pointer-coarse:min-h-11', index === active && 'bg-surface-2']}
		onpointerdown={(event) => event.preventDefault()}
		onpointerenter={() => (active = index)}
		onclick={() => onselect(option)}
	>
		{#if option.kind === 'crew'}
			<Mascot agent={option.agent} size={20} />
			<span class="text-ui text-text">{option.name}</span>
			<span class="min-w-0 flex-1 truncate text-caption text-text-3">{option.role}</span>
		{:else}
			<span class="inline-flex text-text-2"><Icon icon={option.icon ?? File} size={14} /></span>
			<span class="min-w-0 flex-1 truncate font-mono text-caption text-text">{option.name}</span>
			<span class="font-mono text-label text-text-3">{option.where}</span>
		{/if}
		{#if index === active}<span aria-hidden="true" class="font-mono text-label text-text-3">↵</span>{/if}
	</div>
{/snippet}

<div {id} role="listbox" aria-label={m.mention_picker()} class="flex w-75 max-w-full flex-col gap-0.5 rounded-md border border-border-strong bg-surface p-1 shadow-float">
	{#if crew.length}
		<div role="group" aria-labelledby="{id}-crew" class="flex flex-col gap-0.5">
			<p id="{id}-crew" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.mention_crew()}</p>
			{#each crew as option, index (index)}{@render item(option, index)}{/each}
		</div>
	{/if}
	{#if files.length}
		<div role="group" aria-labelledby="{id}-files" class="flex flex-col gap-0.5">
			<p id="{id}-files" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.mention_files()}</p>
			{#each files as option, index (index)}{@render item(option, crew.length + index)}{/each}
		</div>
	{/if}
</div>
