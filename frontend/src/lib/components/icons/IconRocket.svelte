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
	<path d="M5 15c-1.5 1.3-2 4-2 6 2 0 4.7-.5 6-2" />
	<path d="M9 18l-3-3c.8-3.2 2.7-6.6 5.5-9A12 12 0 0 1 21 3a12 12 0 0 1-3 9.5c-2.4 2.8-5.8 4.7-9 5.5Z" />
	<circle cx="15" cy="9" r="1.6" />
</svg>
