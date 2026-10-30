<script lang="ts">
	import type { Snippet } from 'svelte';

	// Design System / Nomi → Overlays / Dialog: the box (surface, border-strong drawn inside, the
	// one shadow), 420 wide (`wide` 440), title and description over the content, actions in a
	// footer. Presentational: Dialog puts it in a native modal; the gallery shows it on the scrim.
	type Props = { id: string; title: string; description?: string; wide?: boolean; children?: Snippet; actions?: Snippet };
	let { id, title, description, wide = false, children, actions }: Props = $props();
</script>

<div class={['flex max-w-full flex-col overflow-hidden rounded-md bg-surface text-text shadow-float outline -outline-offset-1 outline-border-strong', wide ? 'w-110' : 'w-105']}>
	<div class="flex flex-col gap-2 p-5">
		<h2 id="{id}-title" class="text-body font-medium text-text">{title}</h2>
		{#if description}<p id="{id}-description" class="text-ui leading-normal text-text-2">{description}</p>{/if}
		{@render children?.()}
	</div>
	{#if actions}
		<div class="flex flex-wrap items-center justify-end gap-2 border-t border-border px-5 pt-2.75 pb-3">{@render actions()}</div>
	{/if}
</div>
