<script lang="ts">
	import { onMount } from 'svelte';
	import { initViewport } from '$lib/viewport';
	import '../app.css';

	let { children } = $props();

	onMount(() => {
		return initViewport();
	});
</script>

{@render children()}
