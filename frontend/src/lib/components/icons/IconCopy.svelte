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
	<rect x="9" y="9" width="11" height="11" rx="1.5" />
	<path d="M5 15V6a2 2 0 0 1 2-2h9" />
</svg>
