<script lang="ts">
	import { Search as SearchIcon } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Kbd from '../actions/Kbd.svelte';

	// Twin of frontend-new's Search: an M3 search bar with its shortcut. `plain` is the settings
	// nav's, the shortcut as plain mono text instead of a key.
	type Props = { placeholder: string; value?: string; shortcut?: string; label?: string; plain?: boolean; oninput?: (value: string) => void };
	let { placeholder, value = $bindable(''), shortcut, label, plain = false, oninput }: Props = $props();
</script>

<label class="search" class:search--plain={plain}>
	<Icon icon={SearchIcon} size={20} />
	<input type="search" {placeholder} aria-label={label ?? placeholder} bind:value oninput={() => oninput?.(value)} />
	{#if shortcut && plain}<span class="shortcut">{shortcut}</span>{:else if shortcut}<Kbd keys={shortcut} />{/if}
</label>

<style>
	.search {
		display: flex;
		width: 100%;
		height: 48px;
		align-items: center;
		gap: 12px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search--plain {
		height: 40px;
		padding: 0 12px;
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
	}
	input {
		min-width: 0;
		flex: 1;
		border: 0;
		background: transparent;
		outline: none;
		color: var(--md-sys-color-on-surface);
		font: var(--md-sys-typescale-body-large-weight) var(--md-sys-typescale-body-large-size) / var(--md-sys-typescale-body-large-line-height) var(--md-sys-typescale-body-large-font);
	}
	input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.shortcut {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
