<script lang="ts">
	// The "@" list of a soul editor: the crew first, then the catalog's tools. It follows the chat's
	// MentionPicker (same rows, same keys: the editor keeps focus and moves `active`, a click picks
	// without taking focus), which only knows crew and files, so the tools' rows live here.
	import { Wrench } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import type { MentionChoice } from '$lib/mentions';

	// `active` counts in the order shown, crew then tools.
	type Props = { id: string; options: MentionChoice[]; active?: number; onselect: (option: MentionChoice) => void };
	let { id, options, active = $bindable(0), onselect }: Props = $props();

	const crew = $derived(options.filter((option) => option.kind === 'crew'));
	const tools = $derived(options.filter((option) => option.kind === 'tool'));
</script>

{#snippet item(option: MentionChoice, index: number)}
	<!-- Keys are handled by the editor, which keeps focus (aria-activedescendant). -->
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
		{:else if option.kind === 'tool'}
			<span class="inline-flex text-text-2"><Icon icon={Wrench} size={14} /></span>
			<span class="font-mono text-caption text-text">{option.name}</span>
			<span class="min-w-0 flex-1 truncate text-caption text-text-3">{option.description}</span>
		{/if}
		{#if index === active}<span aria-hidden="true" class="font-mono text-label text-text-3">↵</span>{/if}
	</div>
{/snippet}

<div {id} role="listbox" aria-label={m.toolpicker_mention_label()} class="flex w-90 max-w-full flex-col gap-0.5 rounded-md bg-surface outline -outline-offset-1 outline-border-strong p-1 shadow-float">
	{#if crew.length}
		<div role="group" aria-labelledby="{id}-crew" class="flex flex-col gap-0.5">
			<p id="{id}-crew" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.mention_crew()}</p>
			{#each crew as option, index (index)}{@render item(option, index)}{/each}
		</div>
	{/if}
	{#if tools.length}
		<div role="group" aria-labelledby="{id}-tools" class="flex flex-col gap-0.5">
			<p id="{id}-tools" role="presentation" class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{m.mention_tools()}</p>
			{#each tools as option, index (index)}{@render item(option, crew.length + index)}{/each}
		</div>
	{/if}
</div>
