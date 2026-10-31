<!-- With its shortcut: the placeholder says what it searches. -->
<script lang="ts">
	import Search from '$lib/ds/inputs/Search.svelte';

	let query = $state('');
</script>

<div class="w-full max-w-80">
	<Search placeholder="Search chats, memory, files" shortcut="⌘K" bind:value={query} />
</div>
