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
	<path d="M3 12a9 9 0 1 0 3-6.7" />
	<polyline points="3 4 3 9 8 9" />
	<polyline points="12 8 12 12 15 14" />
</svg>
