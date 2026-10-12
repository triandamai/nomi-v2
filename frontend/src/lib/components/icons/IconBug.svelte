<script lang="ts">
	let { size = 20 }: { size?: number } = $props();
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
	<rect x="7" y="7" width="10" height="13" rx="5" />
	<path d="M12 11v9M9 4.5 10.5 7M15 4.5 13.5 7M3 13h4M17 13h4M4 8.5l3 1.5M20 8.5l-3 1.5M4 19l3-2M20 19l-3-2" />
</svg>
