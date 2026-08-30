<script lang="ts">
	import Menu from './Menu.svelte';
	import MenuItem from './MenuItem.svelte';
	import IconChevronDown from '../icons/IconChevronDown.svelte';
	import IconChevronUp from '../icons/IconChevronUp.svelte';

	let {
		label,
		name,
		options,
		value = $bindable(options[0]?.value ?? ''),
		class: extraClass = '',
	}: {
		label: string;
		name: string;
		options: { value: string; label: string }[];
		value?: string;
		class?: string;
	} = $props();

	// `options` must be destructured before `value` above so value's default expression can
	// reference it — matches native <select>'s own behavior of defaulting to the first
	// <option> when nothing is explicitly selected.

	let open = $state(false);
	const selectedLabel = $derived(options.find((o) => o.value === value)?.label ?? '');
</script>

<div class="m3-select {extraClass}">
	<span class="m3-select__label">{label}</span>
	<input type="hidden" {name} {value} />
	<Menu bind:open>
		{#snippet trigger({ toggle })}
			<button type="button" class="m3-select__trigger" onclick={toggle} aria-haspopup="listbox" aria-expanded={open}>
				<span>{selectedLabel}</span>
				{#if open}
					<IconChevronUp size={18} />
				{:else}
					<IconChevronDown size={18} />
				{/if}
			</button>
		{/snippet}
		{#each options as option (option.value)}
			<MenuItem
				selected={option.value === value}
				onclick={() => {
					value = option.value;
					open = false;
				}}
			>
				{option.label}
			</MenuItem>
		{/each}
	</Menu>
</div>

<style>
	.m3-select {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.m3-select__label {
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
		letter-spacing: var(--md-sys-typescale-body-small-tracking);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-select__trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		box-sizing: border-box;
		height: 44px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		cursor: pointer;
	}
	.m3-select__trigger:focus-visible {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 15px;
	}
</style>
