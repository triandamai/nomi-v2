<!-- A message's files over its bubble: each opens in the viewer. -->
<script lang="ts">
	import SentFiles from '$lib/ds/thread/SentFiles.svelte';
</script>

<SentFiles
	files={[
		{ name: 'bca-sep-2026.pdf', meta: 'PDF · 4,2 MB', onopen: () => {} },
		{ name: 'bca-oct-2026.pdf', meta: 'PDF · 3,9 MB', onopen: () => {} },
	]}
/>
