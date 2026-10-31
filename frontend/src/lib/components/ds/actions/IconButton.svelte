<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import M3IconButton from '../../m3/IconButton.svelte';
	import Icon from '../foundations/Icon.svelte';

	// Twin of frontend-new's IconButton on the M3 icon button: outline → outlined, ghost →
	// standard, send → filled. The label is what screen readers say: icon buttons never go unnamed.
	type Props = {
		icon: Component<LucideProps>;
		label: string;
		/** 14 where the pen draws a smaller glyph in the same 30px target (a card header's tools). */
		iconSize?: 16 | 14;
		variant?: 'outline' | 'ghost' | 'send';
		disabled?: boolean;
		type?: 'button' | 'submit';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
		[attribute: `aria-${string}`]: unknown;
	};
	let { icon, label, iconSize = 16, variant = 'outline', disabled = false, type = 'button', onclick, ...rest }: Props = $props();

	const M3_VARIANT = { outline: 'outlined', ghost: 'standard', send: 'filled' } as const;
	// M3's icon sizes: 20 in an icon button, 18 for the smaller glyph.
	const SIZE = { 16: 20, 14: 18 } as const;
</script>

<M3IconButton {type} {onclick} {disabled} variant={M3_VARIANT[variant]} aria-label={label} {...rest}>
	<Icon {icon} size={SIZE[iconSize]} />
</M3IconButton>
