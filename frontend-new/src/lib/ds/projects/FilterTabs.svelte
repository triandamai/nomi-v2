<script lang="ts">
	import FilterTab from './FilterTab.svelte';

	// Design System / Projects → Project / Toolbar's tabs (TnDHg): the filter tabs on a 30 tall
	// surface-2 track (2 in, 2 apart). They narrow one list, so they're a radio group: arrow keys
	// move the choice, as the platform's own do.
	type Option = { value: string; label: string; count?: number | string };
	type Props = { label: string; options: Option[]; value?: string; onchange?: (value: string) => void };
	let { label, options, value = $bindable(options[0]?.value ?? ''), onchange }: Props = $props();

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

<div role="radiogroup" aria-label={label} tabindex="-1" {onkeydown} class="inline-flex max-w-full shrink-0 items-center gap-0.5 rounded-sm bg-surface-2 p-0.5">
	{#each options as option, index (option.value)}
		{@const checked = option.value === value}
		<button
			bind:this={buttons[index]}
			type="button"
			role="radio"
			aria-checked={checked}
			tabindex={checked ? 0 : -1}
			onclick={() => choose(option.value)}
			class="group inline-flex cursor-pointer rounded-sm outline-offset-1 pointer-coarse:min-h-11 pointer-coarse:items-center"
			><FilterTab label={option.label} count={option.count} active={checked} /></button
		>
	{/each}
</div>
