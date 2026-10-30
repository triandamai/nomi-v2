<script lang="ts">
	// Design System / Nomi → Overlays / Tooltip: the inverted tip itself, with the shortcut.
	// Presentational: Tooltip shows it on hover or focus; the gallery shows it as drawn.
	type Props = { label: string; shortcut?: string; id?: string; element?: HTMLElement; [attribute: string]: unknown };
	let { label, shortcut, id, element = $bindable(), ...rest }: Props = $props();
</script>

<span
	bind:this={element}
	{id}
	role="tooltip"
	{...rest}
	class={['pointer-events-none inline-flex items-center gap-2 rounded-sm bg-text px-2 py-1 text-caption text-bg', rest.class as string | undefined]}
>
	{label}
	{#if shortcut}<span class="font-mono text-label text-text-3">{shortcut}</span>{/if}
</span>
