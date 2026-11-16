<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { Check, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Sort Sheet, Workspace Sheet (Flow / Mobile Projects): one choice in a phone's sheet,
	// laid out as a Sheet Row (52 tall, [0,20], 14 apart, the label at 16) with what it is on the left
	// (an icon at 20, or `lead`: a workspace's tile), a second line at 13 where there is one, and the
	// chosen one marked with an accent check at its end. It says whether it's chosen (aria-pressed).
	type Props = { label: string; detail?: string; icon?: Component<LucideProps>; lead?: Snippet; chosen?: boolean; onclick: () => void };
	let { label, detail, icon, lead, chosen = false, onclick }: Props = $props();
</script>

<button type="button" aria-pressed={chosen} class={['flex w-full cursor-pointer items-center gap-3.5 px-5 text-left -outline-offset-2', detail ? 'min-h-15 py-2' : 'h-13']} {onclick}>
	{#if lead}{@render lead()}{:else if icon}<span class="inline-flex shrink-0 text-text-2"><Icon {icon} size={20} /></span>{/if}
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="truncate text-phone text-text">{label}</span>
		{#if detail}<span class="truncate text-ui text-text-3">{detail}</span>{/if}
	</span>
	{#if chosen}<span class="inline-flex shrink-0 text-accent"><Icon icon={Check} size={20} /></span>{/if}
</button>
