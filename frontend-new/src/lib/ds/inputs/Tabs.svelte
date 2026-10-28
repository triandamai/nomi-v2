<script lang="ts">
	import type { Snippet } from 'svelte';

	// Design System / Nomi → Tabs: 40px tall, the open one in text colour with a line under it.
	// Arrow keys move between tabs (and open them, as most apps do).
	type Tab = { value: string; label: string };
	type Props = { label: string; tabs: Tab[]; value?: string; panel?: Snippet<[string]>; onchange?: (value: string) => void };
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

<div role="tablist" aria-label={label} tabindex="-1" {onkeydown} class="flex gap-5 border-b border-border">
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
			onclick={() => open(tab.value)}
			class={[
				'-mb-px inline-flex h-10 cursor-pointer items-center gap-1.5 border-b text-ui outline-offset-1',
				selected ? 'border-text font-medium text-text' : 'border-transparent text-text-3 hover:text-text-2',
			]}>{tab.label}</button
		>
	{/each}
</div>
{#if panel}
	<div role="tabpanel" id="{id}-panel" aria-labelledby="{id}-tab-{tabs.findIndex((tab) => tab.value === value)}" tabindex="0">
		{@render panel(value)}
	</div>
{/if}
