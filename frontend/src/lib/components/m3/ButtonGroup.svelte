<script lang="ts" generics="T extends string">
	// M3 Expressive connected button group: segments sit 2px apart with tight inner corners; the
	// selected segment springs to fully round. Single-select, like a segmented control.

	let {
		options,
		value = $bindable(),
		name,
		'aria-label': ariaLabel,
		onchange,
	}: {
		options: { value: T; label: string }[];
		value: T;
		/** When set, a hidden input submits the current value with the surrounding form. */
		name?: string;
		'aria-label': string;
		onchange?: (value: T) => void;
	} = $props();

	function pick(next: T) {
		value = next;
		onchange?.(next);
	}
</script>

<div class="m3-button-group" role="group" aria-label={ariaLabel}>
	{#each options as option (option.value)}
		<button
			type="button"
			class="m3-button-group__item"
			class:m3-button-group__item--selected={option.value === value}
			aria-pressed={option.value === value}
			onclick={() => pick(option.value)}
		>
			{option.label}
		</button>
	{/each}
</div>
{#if name}
	<input type="hidden" {name} {value} />
{/if}

<style>
	.m3-button-group {
		display: flex;
		gap: 2px;
	}
	.m3-button-group__item {
		flex: 1;
		height: 48px;
		padding: 0 18px;
		border: none;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		cursor: pointer;
		white-space: nowrap;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast),
			color var(--nomi-motion-effects-fast);
	}
	.m3-button-group__item:first-child {
		border-radius: var(--md-sys-shape-corner-full) var(--md-sys-shape-corner-small) var(--md-sys-shape-corner-small)
			var(--md-sys-shape-corner-full);
	}
	.m3-button-group__item:last-child {
		border-radius: var(--md-sys-shape-corner-small) var(--md-sys-shape-corner-full) var(--md-sys-shape-corner-full)
			var(--md-sys-shape-corner-small);
	}
	.m3-button-group__item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, var(--md-sys-color-surface-container-high));
	}
	.m3-button-group__item.m3-button-group__item--selected {
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
</style>
