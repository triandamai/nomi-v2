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
	<rect x="4" y="4" width="7" height="7" rx="1.5" />
	<rect x="13" y="4" width="7" height="7" rx="1.5" />
	<rect x="4" y="13" width="7" height="7" rx="1.5" />
	<rect x="13" y="13" width="7" height="7" rx="1.5" />
</svg>
