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
	<rect x="5" y="8" width="14" height="10" rx="2" />
	<line x1="12" y1="8" x2="12" y2="4" />
	<circle cx="12" cy="3" r="1" />
	<circle cx="9" cy="13" r="1" />
	<circle cx="15" cy="13" r="1" />
</svg>
