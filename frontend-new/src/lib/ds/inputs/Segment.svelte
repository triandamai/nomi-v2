<script lang="ts">
	// Design System / Nomi → Segment: one view in a Segmented control, active (on bg, outlined) or
	// not, with an optional count. Presentational: Segmented makes each one a radio. The stroke sits
	// inside the pen's 12px padding.
	type Props = { label: string; count?: number | string; checked?: boolean };
	let { label, count, checked = false }: Props = $props();
</script>

<span
	class={[
		'inline-flex h-6.5 items-center gap-1.5 rounded-sm border px-2.75 text-ui',
		checked ? 'border-border-strong bg-bg font-medium text-text' : 'border-transparent text-text-2 group-hover:text-text',
	]}
>
	<span>{label}</span>
	{#if count !== undefined}<span class="font-mono text-label font-normal text-text-3">{count}</span>{/if}
</span>
