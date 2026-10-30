<script lang="ts">
	// Design System / Nomi → Tab: 40px tall, the open one in text colour with a line under it.
	// Presentational: Tabs makes each one a tab. The line is drawn inside the 40px, as the pen's
	// stroke is, so the label stays centred.
	type Props = { label: string; selected?: boolean };
	let { label, selected = false }: Props = $props();
</script>

<span
	class={[
		'relative inline-flex h-10 items-center gap-1.5 text-ui',
		selected ? 'font-medium text-text after:absolute after:inset-x-0 after:bottom-0 after:h-px after:bg-text' : 'text-text-3 group-hover:text-text-2',
	]}>{label}</span
>
