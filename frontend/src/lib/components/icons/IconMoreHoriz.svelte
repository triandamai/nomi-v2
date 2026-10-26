<script lang="ts">
	let { size = 20 }: { size?: number } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2.5"
	stroke-linecap="round"
	aria-hidden="true"
	focusable="false"
>
	<line x1="6" y1="12" x2="6.01" y2="12" />
	<line x1="12" y1="12" x2="12.01" y2="12" />
	<line x1="18" y1="12" x2="18.01" y2="12" />
</svg>
