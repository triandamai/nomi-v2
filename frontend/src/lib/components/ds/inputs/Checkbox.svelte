<script lang="ts">
	import CheckMark from './CheckMark.svelte';

	// A real checkbox with its text label.
	type Props = { label: string; checked?: boolean; disabled?: boolean; name?: string; onchange?: (checked: boolean) => void };
	let { label, checked = $bindable(false), disabled = false, name, onchange }: Props = $props();
</script>

<label class="checkbox" class:checkbox--disabled={disabled}>
	<CheckMark bind:checked {disabled} {name} {onchange} />
	{label}
</label>

<style>
	.checkbox {
		display: inline-flex;
		align-items: center;
		gap: 12px;
		cursor: pointer;
		font: var(--md-sys-typescale-body-large-weight) var(--md-sys-typescale-body-large-size) / var(--md-sys-typescale-body-large-line-height) var(--md-sys-typescale-body-large-font);
		color: var(--md-sys-color-on-surface);
	}
	.checkbox--disabled {
		cursor: default;
		opacity: 0.38;
	}
</style>
