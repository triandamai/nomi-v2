<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * A board inside a sheet section, as in the pen file: its name, a note in mono, the examples.
	 * The Build sheet puts the note right after the name, tops aligned (`inline`); the others
	 * right-align it, centred on the name.
	 * `phone` boards are 430 wide, a 390pt phone plus the board's padding (Build on mobile).
	 */
	type Props = { title: string; note: string; inline?: boolean; phone?: boolean; children: Snippet };
	let { title, note, inline = false, phone = false, children }: Props = $props();
</script>

<div data-board class={['flex min-w-0 flex-col gap-5 rounded-md border border-border p-5.75', phone && 'w-107.5 max-w-full shrink-0']}>
	<p class={['flex gap-3 text-ui font-medium text-text', inline ? 'items-start' : 'items-center justify-between']}>
		<span class="shrink-0">{title}</span> <span class={['min-w-0 font-mono text-label text-text-3', !inline && 'text-right']}>{note}</span>
	</p>
	{@render children()}
</div>
