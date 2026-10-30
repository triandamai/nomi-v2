<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * A board inside a sheet section, as in the pen file: its name, a note in mono, the examples.
	 * Chat responses boards right-align the note, centred on the name, 20 above the examples. The
	 * Build sheet puts it right after the name, tops aligned (`inline`); the Nomi sheet right after
	 * the name too, centred, 16 above the examples (`nomi`).
	 * `phone` boards are 430 wide, a 390pt phone plus the board's padding (Build on mobile);
	 * `spacious` ones put 28 between their parts (the provider error boards).
	 */
	type Props = { title: string; note: string; inline?: boolean; nomi?: boolean; phone?: boolean; spacious?: boolean; children: Snippet };
	let { title, note, inline = false, nomi = false, phone = false, spacious = false, children }: Props = $props();
</script>

<div data-board class={['flex min-w-0 flex-col rounded-md border border-border p-5.75', nomi ? 'gap-4' : spacious ? 'gap-7' : 'gap-5', phone && 'w-107.5 max-w-full shrink-0']}>
	<p class={['flex gap-3 text-ui font-medium text-text', inline ? 'items-start' : 'items-center', !inline && !nomi && 'justify-between']}>
		<span class="shrink-0">{title}</span> <span class={['min-w-0 font-mono text-label text-text-3', !inline && !nomi && 'text-right']}>{note}</span>
	</p>
	{@render children()}
</div>
