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
	<circle cx="18" cy="5" r="2.5" />
	<circle cx="6" cy="12" r="2.5" />
	<circle cx="18" cy="19" r="2.5" />
	<line x1="8.3" y1="10.7" x2="15.7" y2="6.3" />
	<line x1="8.3" y1="13.3" x2="15.7" y2="17.7" />
</svg>
