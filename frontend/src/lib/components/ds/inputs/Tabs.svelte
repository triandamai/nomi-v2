<script lang="ts">
	import type { Snippet } from 'svelte';
	import Tab from './Tab.svelte';

	// Twin of frontend-new's Tabs on M3 primary tabs. Arrow keys move between tabs (and open them).
	type TabItem = { value: string; label: string };
	type Props = { label: string; tabs: TabItem[]; value?: string; panel?: Snippet<[string]>; onchange?: (value: string) => void };
	let { label, tabs, value = $bindable(tabs[0]?.value ?? ''), panel, onchange }: Props = $props();

	const id = $props.id();
	let buttons: HTMLButtonElement[] = $state([]);

	function open(next: string, focus = false) {
		value = next;
		onchange?.(next);
		if (focus) buttons[tabs.findIndex((tab) => tab.value === next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		const at = tabs.findIndex((tab) => tab.value === value);
		const next =
			event.key === 'ArrowRight' ? at + 1 : event.key === 'ArrowLeft' ? at - 1 : event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : null;
		if (next === null) return;
		event.preventDefault();
		open(tabs[(next + tabs.length) % tabs.length].value, true);
	}
</script>

<div role="tablist" aria-label={label} tabindex="-1" {onkeydown} class="tabs">
	{#each tabs as tab, index (tab.value)}
		{@const selected = tab.value === value}
		<button
			bind:this={buttons[index]}
			type="button"
			role="tab"
			id="{id}-tab-{index}"
			aria-selected={selected}
			aria-controls="{id}-panel"
			tabindex={selected ? 0 : -1}
			onclick={() => open(tab.value)}><Tab label={tab.label} {selected} /></button
		>
	{/each}
</div>
{#if panel}
	<div role="tabpanel" id="{id}-panel" aria-labelledby="{id}-tab-{tabs.findIndex((tab) => tab.value === value)}" tabindex="0">
		{@render panel(value)}
	</div>
{/if}

<style>
	.tabs {
		display: flex;
		overflow-x: auto;
		border-bottom: 1px solid var(--md-sys-color-surface-container-highest);
	}
	button {
		display: inline-flex;
		padding: 0;
		border: 0;
		background: none;
		cursor: pointer;
	}
	button:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	button:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
</style>
