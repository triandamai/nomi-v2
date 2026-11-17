<!-- On a shared link: locked, named only, nothing to open. -->
<script lang="ts">
	import SentFiles from '$lib/ds/thread/SentFiles.svelte';
</script>

<SentFiles
	files={[
		{ name: 'bca-sep-2026.pdf', meta: 'PDF · private', locked: true },
		{ name: 'bca-oct-2026.pdf', meta: 'PDF · private', locked: true },
	]}
/>
