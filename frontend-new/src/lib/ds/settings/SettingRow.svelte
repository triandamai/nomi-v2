<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / General (m7lvY1) → Row: one setting, its name and what it means (in mono when
	// it's a machine's value: an address, a format), and its control at the end, 14 apart, over the
	// row's bottom stroke. `danger` names a setting that can't be undone (Delete account).
	type Props = { title: string; description?: string; mono?: boolean; danger?: boolean; control?: Snippet };
	let { title, description, mono = false, danger = false, control }: Props = $props();
</script>

<div class="flex items-center gap-3.5 border-b border-border pt-2.75 pb-2.5">
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class={['text-ui font-medium', danger ? 'text-danger' : 'text-text']}>{title}</span>
		{#if description}<span class={['text-text-3', mono ? 'font-mono text-label' : 'text-caption']}>{description}</span>{/if}
	</div>
	{#if control}<div class="flex shrink-0 items-center gap-2">{@render control()}</div>{/if}
</div>
