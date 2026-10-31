<script lang="ts">
	import { Check } from '@lucide/svelte';

	// The M3 checkbox box alone, for rows that bring their own label (a selection list, a
	// many-of-many question). Checkbox wraps it with a text label.
	type Props = {
		checked?: boolean;
		disabled?: boolean;
		name?: string;
		id?: string;
		onchange?: (checked: boolean) => void;
		[attribute: `aria-${string}`]: unknown;
	};
	let { checked = $bindable(false), disabled = false, name, id, onchange, ...rest }: Props = $props();
</script>

<span class="box" class:box--disabled={disabled}>
	<input type="checkbox" {id} {name} {disabled} bind:checked onchange={() => onchange?.(checked)} {...rest} />
	<span class="indicator" class:indicator--checked={checked}>
		{#if checked}<Check size={14} strokeWidth={2.5} aria-hidden="true" />{/if}
	</span>
</span>

<style>
	.box {
		position: relative;
		display: inline-flex;
		width: 18px;
		height: 18px;
		flex-shrink: 0;
	}
	.box input {
		position: absolute;
		inset: -8px;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	@media (pointer: coarse) {
		.box input {
			inset: -13px;
		}
	}
	.box--disabled {
		opacity: 0.38;
	}
	.box--disabled input {
		cursor: default;
	}
	.indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: 7px;
		border: 2px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-primary);
		pointer-events: none;
		transition:
			background-color var(--nomi-motion-effects-fast),
			border-color var(--nomi-motion-effects-fast);
	}
	.box input:hover ~ .indicator {
		border-color: var(--md-sys-color-on-surface);
	}
	.indicator--checked,
	.box input:hover ~ .indicator--checked {
		background: var(--md-sys-color-primary);
		border-color: var(--md-sys-color-primary);
	}
	.box input:focus-visible ~ .indicator {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
