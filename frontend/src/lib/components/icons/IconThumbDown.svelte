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
	<path d="M17 13V4h3v9h-3zm0 0-4 8a2 2 0 0 1-2-2v-4H6.5a2 2 0 0 1-1.94-2.5l1.6-6A2 2 0 0 1 8.1 4H17" />
</svg>
