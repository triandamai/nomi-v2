<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Nomi → Actions: a 30×30 hit area with a 16px icon. The label is what screen
	// readers say (and the tooltip, where one is added): icon buttons never go unnamed.
	type Props = {
		icon: Component<LucideProps>;
		label: string;
		variant?: 'outline' | 'ghost' | 'send';
		disabled?: boolean;
		type?: 'button' | 'submit';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
		[attribute: `aria-${string}`]: unknown;
	};
	let { icon, label, variant = 'outline', disabled = false, type = 'button', onclick, ...rest }: Props = $props();
</script>

<button
	{type}
	{onclick}
	{disabled}
	aria-label={label}
	{...rest}
	class={[
		'inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm outline-offset-1 transition-colors disabled:cursor-default pointer-coarse:size-11 data-focus:outline-2 data-focus:outline-focus',
		variant === 'outline' && 'border border-border-strong text-text-2 hover:enabled:bg-surface-2 hover:enabled:text-text data-hover:bg-surface-2',
		variant === 'ghost' && 'text-text-3 hover:enabled:bg-surface-2 hover:enabled:text-text data-hover:bg-surface-2',
		variant === 'send' && 'bg-accent text-accent-ink disabled:bg-surface-2 disabled:text-text-3',
	]}
>
	<Icon {icon} />
</button>
