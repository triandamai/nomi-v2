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
	/* Corners are real lengths (half the 48px height for "round"), never corner-full: radius then
	   animates smoothly between states instead of snapping or flashing square mid-spring. */
	.m3-button-group {
		--round: 24px;
		--inner: var(--md-sys-shape-corner-small);
		display: flex;
		gap: 2px;
	}
	.m3-button-group__item {
		flex: 1;
		height: 48px;
		padding: 0 18px;
		border: none;
		border-radius: var(--inner);
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
	/* The group reads as one pill: its outer ends are round. */
	.m3-button-group__item:first-child {
		border-radius: var(--round) var(--inner) var(--inner) var(--round);
	}
	.m3-button-group__item:last-child {
		border-radius: var(--inner) var(--round) var(--round) var(--inner);
	}
	.m3-button-group__item:only-child {
		border-radius: var(--round);
	}
	.m3-button-group__item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, var(--md-sys-color-surface-container-high));
	}
	.m3-button-group__item:active {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, var(--md-sys-color-surface-container-high));
	}
	/* Selected springs to fully round, wherever it sits in the group. */
	.m3-button-group .m3-button-group__item--selected {
		border-radius: var(--round);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.m3-button-group .m3-button-group__item--selected:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-primary) 8%, var(--md-sys-color-primary));
	}
</style>
