<script lang="ts">
	let { size = 20, open = true }: { size?: number; open?: boolean } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
	focusable="false"
>
	<rect x="3" y="4" width="18" height="16" rx="3" />
	<line x1="14" y1="4" x2="14" y2="20" />
	{#if open}<rect x="14" y="4" width="7" height="16" rx="0" fill="currentColor" opacity="0.25" stroke="none" />{/if}
</svg>
