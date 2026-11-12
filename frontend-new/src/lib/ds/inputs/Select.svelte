<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Select: one choice from a list, as a 30px field (border-
	// strong, [0,10], the value at 12px and a chevron). It's the platform's own select underneath, so
	// the list, the keyboard and screen readers work as people expect. `placeholder` shows while
	// nothing is chosen. It fills what it's given: the page sets its width (the platform's would be
	// the longest option's).
	type Option = { value: string; label: string };
	type Props = { label: string; options: Option[]; value?: string; placeholder?: string; mono?: boolean; disabled?: boolean; onchange?: (value: string) => void };
	let { label, options, value = $bindable(''), placeholder, mono = false, disabled = false, onchange }: Props = $props();
</script>

<span class="relative inline-flex w-full max-w-full">
	<select
		aria-label={label}
		bind:value
		{disabled}
		onchange={() => onchange?.(value)}
		class={[
			'h-7.5 w-full max-w-full cursor-pointer appearance-none truncate rounded-sm border border-border-strong bg-bg pr-7.25 pl-2.25 text-caption text-text outline-offset-1 hover:enabled:bg-surface disabled:cursor-default disabled:opacity-50',
			mono && 'font-mono',
		]}
	>
		{#if placeholder}<option value="" disabled>{placeholder}</option>{/if}
		{#each options as option (option.value)}<option value={option.value}>{option.label}</option>{/each}
	</select>
	<span class="pointer-events-none absolute top-2.25 right-2.5 inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
</span>
