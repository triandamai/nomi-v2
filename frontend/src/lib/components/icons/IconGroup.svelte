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
	<circle cx="9" cy="8" r="3.2" />
	<path d="M3 19.5c.6-3.2 3-5 6-5s5.4 1.8 6 5" />
	<path d="M15.5 5.2a3 3 0 0 1 0 5.6M17.5 14.8c1.8.6 3 2.2 3.5 4.7" />
</svg>
