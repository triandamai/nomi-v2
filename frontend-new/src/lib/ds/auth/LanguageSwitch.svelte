<script lang="ts">
	import { ChevronDown, Languages } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Auth / Sign in or create → Top / Language: the language in use in a quiet 28px pill. The
	// platform's own picker opens from it (a transparent select over the pill), so it works by
	// keyboard and as a sheet on a phone. 44 high on a phone.
	type Option = { value: string; label: string };
	type Props = {
		value: string;
		options: Option[];
		onchange: (value: string) => void;
		/** The picker's accessible name. */
		label?: string;
	};
	let { value, options, onchange, label = m.language_label() }: Props = $props();

	const current = $derived(options.find((option) => option.value === value)?.label ?? value);
</script>

<label
	class="relative inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-sm border border-border px-2.25 text-text-3 outline-offset-2 transition-colors hover:bg-surface-2 has-focus-visible:outline-2 has-focus-visible:outline-focus max-sm:h-11"
>
	<Icon icon={Languages} size={14} />
	<span class="text-caption text-text-2" aria-hidden="true">{current}</span>
	<Icon icon={ChevronDown} size={12} />
	<select
		aria-label={label}
		{value}
		onchange={(event) => onchange(event.currentTarget.value)}
		class="absolute inset-0 cursor-pointer appearance-none opacity-0 outline-none"
	>
		{#each options as option (option.value)}
			<option value={option.value}>{option.label}</option>
		{/each}
	</select>
</label>
