<script lang="ts">
	// Mobile / Filter Chip, Active and Default (Flow / Mobile Projects — Find & filter): the page's
	// filters in a row that scrolls sideways, [0,16,12,16], 8 apart (46 tall). Each chip is 34 tall and round,
	// [0,14], its name at 14 / 500 and its count in mono at 12, 6 apart: the chosen one filled with
	// text (its words in the page colour), the rest stroked border-strong with the count in text-3.
	// A radio group: the arrow keys move between them.
	type Option = { value: string; label: string; count?: number };
	type Props = { label: string; options: Option[]; value: string; onchange: (value: string) => void };
	let { label, options, value, onchange }: Props = $props();

	let group: HTMLElement | undefined = $state();
	function onkeydown(event: KeyboardEvent) {
		const at = options.findIndex((option) => option.value === value);
		const step = event.key === 'ArrowRight' || event.key === 'ArrowDown' ? 1 : event.key === 'ArrowLeft' || event.key === 'ArrowUp' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const next = options[(at + step + options.length) % options.length];
		onchange(next.value);
		requestAnimationFrame(() => group?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus());
	}
</script>

<div bind:this={group} role="radiogroup" aria-label={label} tabindex="-1" class="flex gap-2 overflow-x-auto px-4 pb-3" {onkeydown}>
	{#each options as option (option.value)}
		{@const chosen = option.value === value}
		<button
			type="button"
			role="radio"
			aria-checked={chosen}
			tabindex={chosen ? 0 : -1}
			class={[
				'inline-flex h-8.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-full text-phone-sm font-medium whitespace-nowrap outline-offset-2',
				chosen ? 'bg-text px-3.5 text-bg' : 'border border-border-strong px-3.25 text-text',
			]}
			onclick={() => onchange(option.value)}
		>
			{option.label}
			{#if option.count !== undefined}<span class={['font-mono text-caption font-normal', chosen ? 'text-bg' : 'text-text-3']}>{option.count}</span>{/if}
		</button>
	{/each}
</div>
