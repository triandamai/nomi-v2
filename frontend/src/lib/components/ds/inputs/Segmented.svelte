<script lang="ts">
	import Segment from './Segment.svelte';

	// Twin of frontend-new's Segmented: an M3 segmented button for a few mutually exclusive views
	// of one list. A radio group: arrow keys move the choice, like the platform's own.
	type Option = { value: string; label: string; count?: number | string };
	type Props = { label: string; options: Option[]; value?: string; surface?: 'filled' | 'bg' | 'surface'; onchange?: (value: string) => void };
	let { label, options, value = $bindable(options[0]?.value ?? ''), surface = 'filled', onchange }: Props = $props();

	let buttons: HTMLButtonElement[] = $state([]);

	function choose(next: string, focus = false) {
		value = next;
		onchange?.(next);
		if (focus) buttons[options.findIndex((option) => option.value === next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		const step = event.key === 'ArrowRight' || event.key === 'ArrowDown' ? 1 : event.key === 'ArrowLeft' || event.key === 'ArrowUp' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const at = options.findIndex((option) => option.value === value);
		choose(options[(at + step + options.length) % options.length].value, true);
	}
</script>

<div role="radiogroup" aria-label={label} tabindex="-1" {onkeydown} class="segmented segmented--{surface}">
	{#each options as option, index (option.value)}
		{@const checked = option.value === value}
		<button bind:this={buttons[index]} type="button" role="radio" aria-checked={checked} tabindex={checked ? 0 : -1} onclick={() => choose(option.value)}>
			<Segment label={option.label} count={option.count} {checked} />
		</button>
	{/each}
</div>

<style>
	.segmented {
		display: inline-flex;
		max-width: 100%;
		align-self: flex-start;
		overflow: hidden;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-full);
	}
	.segmented--bg {
		background: var(--nomi-color-ground);
	}
	.segmented--surface {
		background: var(--md-sys-color-surface-container-lowest);
	}
	button {
		display: inline-flex;
		padding: 0;
		border: 0;
		background: none;
		cursor: pointer;
	}
	button + button {
		border-left: 1px solid var(--md-sys-color-outline);
	}
	button:hover :global(.segment:not(.segment--checked)) {
		background: var(--md-sys-color-surface-container-high);
	}
	button:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	@media (pointer: coarse) {
		button {
			min-height: 44px;
		}
	}
</style>
