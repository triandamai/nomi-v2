<script lang="ts">
	// Design System / Nomi → Status: how much of something is used, as a thin bar and in words
	// ("21.8K / 1M tokens this month"). The real figure, never more than full.
	// `roomy` spaces it 10px apart with the head centred, as in the sidebar's usage card; on its own
	// it's 6px, the head top-aligned.
	type Props = { label: string; value: number; caption?: string; roomy?: boolean };
	let { label, value, caption, roomy = false }: Props = $props();

	const percent = $derived(Math.round(Math.min(Math.max(value, 0), 1) * 100));
	const id = $props.id();
</script>

<div
	role="meter"
	aria-labelledby="{id}-label"
	aria-valuemin={0}
	aria-valuemax={100}
	aria-valuenow={percent}
	aria-valuetext={caption ? `${percent}% · ${caption}` : `${percent}%`}
	class={['flex w-full flex-col', roomy ? 'gap-2.5' : 'gap-1.5']}
>
	<div class={['flex justify-between', roomy ? 'items-center' : 'items-start']}>
		<span id="{id}-label" class="text-ui font-medium text-text">{label}</span>
		<span class="font-mono text-label text-text-3">{percent}%</span>
	</div>
	<div data-track class="h-1 w-full overflow-hidden rounded-full bg-surface-2">
		<div data-fill class="h-full rounded-full bg-text-2" style="width: {Math.min(Math.max(value, 0), 1) * 100}%"></div>
	</div>
	{#if caption}<span class="font-mono text-label text-text-3">{caption}</span>{/if}
</div>
