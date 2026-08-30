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
	<path d="M7 11v9H4v-9h3zm0 0 4-8a2 2 0 0 1 2 2v4h4.5a2 2 0 0 1 1.94 2.5l-1.6 6A2 2 0 0 1 15.9 20H7" />
</svg>
