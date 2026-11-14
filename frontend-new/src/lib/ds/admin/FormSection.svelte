<script lang="ts">
	import type { Snippet } from 'svelte';

	// Admin → Form section: one group of settings or one form, as a flat bordered card: its name and
	// what it changes, the fields, and its actions in a footer under a divider (as the Dialog's box
	// draws them), so each part of a page saves on its own. `danger` outlines what can't be undone.
	// Presentational: the page puts it inside its <form>.
	type Props = {
		title: string;
		description?: string;
		/** Extra state beside the title: a source, a status. */
		aside?: Snippet;
		children?: Snippet;
		footer?: Snippet;
		danger?: boolean;
	};
	let { title, description, aside, children, footer, danger = false }: Props = $props();

	const id = $props.id();
</script>

<section aria-labelledby="{id}-title" class={['flex w-full min-w-0 flex-col rounded-md bg-bg outline -outline-offset-1', danger ? 'outline-danger-border' : 'outline-border']}>
	<div class="flex flex-col gap-4 p-5">
		<div class="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
			<div class="flex min-w-0 flex-1 flex-col gap-1">
				<h2 id="{id}-title" class={['text-ui font-medium', danger ? 'text-danger' : 'text-text']}>{title}</h2>
				{#if description}<p class="max-w-160 text-caption text-text-2">{description}</p>{/if}
			</div>
			{#if aside}<div class="flex shrink-0 items-center gap-2">{@render aside()}</div>{/if}
		</div>
		{@render children?.()}
	</div>
	{#if footer}
		<div class="flex flex-wrap items-center justify-end gap-2 border-t border-border px-5 pt-2.75 pb-3">{@render footer()}</div>
	{/if}
</section>
