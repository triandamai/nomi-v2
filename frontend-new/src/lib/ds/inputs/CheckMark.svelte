<script lang="ts">
	import { Check } from '@lucide/svelte';

	// The 16px box of Design System / Nomi → Check, for rows that bring their own label (a
	// selection list, a many-of-many question). Checkbox wraps it with a text label.
	type Props = {
		checked?: boolean;
		disabled?: boolean;
		name?: string;
		onchange?: (checked: boolean) => void;
		[attribute: `aria-${string}`]: unknown;
	};
	let { checked = $bindable(false), disabled = false, name, onchange, ...rest }: Props = $props();
</script>

<span class="relative inline-flex size-4 shrink-0">
	<input
		type="checkbox"
		{name}
		{disabled}
		bind:checked
		onchange={() => onchange?.(checked)}
		{...rest}
		class="peer size-4 cursor-pointer appearance-none rounded-xs border border-border-strong bg-surface outline-offset-2 checked:border-text checked:bg-text disabled:cursor-default"
	/>
	<Check size={12} strokeWidth={2} aria-hidden="true" class="pointer-events-none absolute top-0.5 left-0.5 hidden text-bg peer-checked:block" />
</span>
