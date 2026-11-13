<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronDown } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Select: one choice from a list, as a 30px field (border-
	// strong, [0,10], the value at 12px and a chevron). It's the platform's own select underneath, so
	// the list, the keyboard and screen readers work as people expect. `placeholder` shows while
	// nothing is chosen. It fills what it's given: the page sets its width (the platform's would be
	// the longest option's). `tall` is the 34px field on the surface Models & providers draws, with
	// `lead` (a mascot) before the value.
	type Option = { value: string; label: string };
	type Props = { label: string; options: Option[]; value?: string; placeholder?: string; mono?: boolean; tall?: boolean; lead?: Snippet; disabled?: boolean; onchange?: (value: string) => void };
	let { label, options, value = $bindable(''), placeholder, mono = false, tall = false, lead, disabled = false, onchange }: Props = $props();
</script>

<span class="relative inline-flex w-full max-w-full">
	<select
		aria-label={label}
		bind:value
		{disabled}
		onchange={() => onchange?.(value)}
		class={[
			'w-full max-w-full cursor-pointer appearance-none truncate rounded-sm border border-border-strong pr-7.25 text-text outline-offset-1 disabled:cursor-default disabled:opacity-50',
			tall ? 'h-8.5 bg-surface text-ui hover:enabled:bg-surface-2' : 'h-7.5 bg-bg text-caption hover:enabled:bg-surface',
			lead ? 'pl-8.5' : 'pl-2.25',
			mono && 'font-mono',
		]}
	>
		{#if placeholder}<option value="" disabled>{placeholder}</option>{/if}
		{#each options as option (option.value)}<option value={option.value}>{option.label}</option>{/each}
	</select>
	{#if lead}<span class="pointer-events-none absolute top-0 left-1.75 flex h-full items-center">{@render lead()}</span>{/if}
	<span class={['pointer-events-none absolute right-2.5 inline-flex text-text-3', tall ? 'top-2.75' : 'top-2.25']}><Icon icon={ChevronDown} size={12} /></span>
</span>
