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
	<circle cx="6" cy="7" r="2.5" /><circle cx="18" cy="9" r="2.5" /><circle cx="10" cy="18" r="2.5" /><path d="M8.3 8 15.6 8.6M7 9.3l2.2 6.4M16.4 11l-4.4 5.4" />
</svg>
