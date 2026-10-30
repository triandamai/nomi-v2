<script lang="ts">
	import { Check } from '@lucide/svelte';

	// Design System / Nomi → Check: a real checkbox, 16px, filled in text colour when checked.
	type Props = { label: string; checked?: boolean; disabled?: boolean; name?: string; onchange?: (checked: boolean) => void };
	let { label, checked = $bindable(false), disabled = false, name, onchange }: Props = $props();
</script>

<label class={['inline-flex cursor-pointer items-center gap-2 text-ui text-text', disabled && 'cursor-default opacity-40']}>
	<span class="relative inline-flex size-4 shrink-0">
		<input
			type="checkbox"
			{name}
			{disabled}
			bind:checked
			onchange={() => onchange?.(checked)}
			class="peer size-4 cursor-pointer appearance-none rounded-xs border border-border-strong bg-surface outline-offset-2 checked:border-text checked:bg-text disabled:cursor-default"
		/>
		<Check size={12} strokeWidth={2} aria-hidden="true" class="pointer-events-none absolute top-0.5 left-0.5 hidden text-bg peer-checked:block" />
	</span>
	{label}
</label>
