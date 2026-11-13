<script lang="ts">
	// App / Settings / Workspaces (l222Mz) → Tile: a workspace's initial on its tone, 10 round: 40
	// on a card (15/600), 56 on the workspace's own page (28/600). The tones are the switcher's.
	type Props = { initial: string; tone: 'plain' | 'teal' | 'amber'; size?: 40 | 56 };
	let { initial, tone, size = 40 }: Props = $props();
</script>

<span
	aria-hidden="true"
	class={[
		'inline-flex shrink-0 items-center justify-center rounded-md font-semibold',
		size === 56 ? 'size-14 text-display' : 'size-10 text-body',
		tone === 'plain' && 'bg-surface-2 text-text',
		tone === 'teal' && 'bg-tile-teal text-tile-ink',
		tone === 'amber' && 'bg-tile-amber text-tile-ink',
	]}>{initial}</span
>
