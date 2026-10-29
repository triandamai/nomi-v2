<script lang="ts">
	// Build together: a person in a shared project, as initials. Each teammate wears their peer
	// colour; you stay neutral. `ring` marks someone present right now.
	type Props = { initials: string; peer?: 1 | 2; size?: 28 | 20; ring?: boolean };
	let { initials, peer, size = 28, ring = false }: Props = $props();
</script>

<span
	data-avatar
	class={[
		'inline-flex shrink-0 items-center justify-center rounded-full font-mono',
		size === 28 ? 'size-7 text-label' : 'size-5 text-nano',
		peer === 1 && 'bg-peer-1-bg text-peer-1',
		peer === 2 && 'bg-peer-2-bg text-peer-2',
		!peer && 'bg-surface-2 text-text-2',
		ring && peer === 1 && 'border-2 border-peer-1',
		ring && peer === 2 && 'border-2 border-peer-2',
		ring && !peer && 'border-2 border-bg',
	]}>{initials}</span
>
