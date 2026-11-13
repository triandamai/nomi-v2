<!-- A memory the crew uses often. -->
<script lang="ts">
	import MemoryEntry from '$lib/ds/settings/MemoryEntry.svelte';
</script>

<ul class="w-full max-w-205 rounded-md bg-surface outline -outline-offset-1 outline-border">
	<MemoryEntry
		content="Short, direct answers; no emoji"
		strength={{ level: 4, name: 'strong' }}
		meta="used 12× · last 21 Oct"
		labels={{ edit: 'Correct it', forget: 'Forget it', save: 'Save', cancel: 'Cancel', field: 'What Nomi should remember' }}
	/>
</ul>
