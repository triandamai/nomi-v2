<script lang="ts">
	// Design System / Nomi → Segmented: a few mutually exclusive views of one list ("All 4 · Live 2
	// · Building 1"). A radio group: arrow keys move the choice, like the platform's own.
	type Option = { value: string; label: string; count?: number | string };
	// `surface`: filled (surface-2) on its own, or outlined on bg or surface where the pen puts it
	// inside a card (Share, a form).
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

<div role="radiogroup" aria-label={label} tabindex="-1" {onkeydown} class={[
		'inline-flex max-w-full flex-wrap gap-0.5 self-start rounded-sm p-0.5',
		surface === 'filled' && 'bg-surface-2',
		surface === 'bg' && 'border border-border bg-bg',
		surface === 'surface' && 'border border-border bg-surface',
	]}>
	{#each options as option, index (option.value)}
		{@const checked = option.value === value}
		<button
			bind:this={buttons[index]}
			type="button"
			role="radio"
			aria-checked={checked}
			tabindex={checked ? 0 : -1}
			onclick={() => choose(option.value)}
			class={[
				'inline-flex h-6.5 cursor-pointer items-center gap-1.5 rounded-sm border px-3 text-ui outline-offset-1 pointer-coarse:min-h-11',
				checked ? 'border-border-strong bg-bg font-medium text-text' : 'border-transparent text-text-2 hover:text-text',
			]}
		>
			<span>{option.label}</span>
			{#if option.count !== undefined}<span class="font-mono text-label font-normal text-text-3">{option.count}</span>{/if}
		</button>
	{/each}
</div>
