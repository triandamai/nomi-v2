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
	<line x1="4" y1="7" x2="11" y2="7" />
	<line x1="15" y1="7" x2="20" y2="7" />
	<circle cx="13" cy="7" r="2" />
	<line x1="4" y1="17" x2="7" y2="17" />
	<line x1="11" y1="17" x2="20" y2="17" />
	<circle cx="9" cy="17" r="2" />
</svg>
