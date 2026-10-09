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
	<polyline points="17 18 11 12 17 6" /><line x1="7" y1="6" x2="7" y2="18" />
</svg>
