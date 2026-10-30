<script lang="ts">
	import Segment from './Segment.svelte';

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
		surface === 'bg' && 'bg-bg outline -outline-offset-1 outline-border',
		surface === 'surface' && 'bg-surface outline -outline-offset-1 outline-border',
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
			class="group inline-flex cursor-pointer items-center rounded-sm outline-offset-1 pointer-coarse:min-h-11"
		>
			<Segment label={option.label} count={option.count} {checked} />
		</button>
	{/each}
</div>
