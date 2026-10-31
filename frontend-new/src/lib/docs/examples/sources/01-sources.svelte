<!-- Sources: numbered like the citations, each with where it came from. -->
<script lang="ts">
	import Sources from '$lib/ds/chat/Sources.svelte';
</script>

<div class="w-full max-w-170">
	<Sources
		id="answer-sources"
		sources={[
			{ title: 'Peraturan Kos Mawar 2026.pdf', detail: 'attachment · read by Maya', kind: 'file' },
			{ title: 'Kos Mawar Tebet — kamar AC', detail: 'kosku.id', kind: 'web' },
		]}
	/>
</div>
