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
	<circle cx="12" cy="5" r="1.5" fill="currentColor" stroke="none" />
	<circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none" />
	<circle cx="12" cy="19" r="1.5" fill="currentColor" stroke="none" />
</svg>
