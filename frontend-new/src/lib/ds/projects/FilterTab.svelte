<script lang="ts">
	// Design System / Projects → Project / Filter Tab / Active (Tgc5G) and Default (kTRZ9): one
	// filter of the projects list with its count. 26 tall, 10 in (the stroke inside), regular weight
	// in both states; the open one sits on the page colour with a strong stroke. Presentational:
	// FilterTabs makes each one a radio.
	type Props = { label: string; count?: number | string; active?: boolean };
	let { label, count, active = false }: Props = $props();
</script>

<span
	class={[
		'inline-flex h-6.5 items-center gap-1.5 rounded-sm border px-2.25 text-ui whitespace-nowrap',
		active ? 'border-border-strong bg-bg text-text' : 'border-transparent text-text-2 group-hover:text-text',
	]}
>
	<span>{label}</span>
	{#if count !== undefined}<span class="font-mono text-label text-text-3">{count}</span>{/if}
</span>
