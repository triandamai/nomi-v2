<!-- A strong topic, closed. -->
<script lang="ts">
	import MemoryTable from '$lib/ds/settings/MemoryTable.svelte';
	import MemoryTopicRow from '$lib/ds/settings/MemoryTopicRow.svelte';
</script>

<div class="w-full max-w-205">
	<MemoryTable columns={{ topic: 'Topic', knows: 'What Nomi knows', strength: 'Strength', updated: 'Updated' }} note="Every memory starts at Steady.">
		<MemoryTopicRow name="Preferences" count="8 memories" summary="English and Bahasa; short, direct answers; no emoji" strength={{ level: 5, name: 'core' }} updated="21 Oct" />
	</MemoryTable>
</div>
