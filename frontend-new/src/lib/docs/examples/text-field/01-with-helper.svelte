<!-- With a helper: what is allowed, and what it becomes. Mono for a slug. -->
<script lang="ts">
	import TextField from '$lib/ds/inputs/TextField.svelte';

	let name = $state('budget-keluarga');
</script>

<div class="w-full max-w-80">
	<TextField label="Project name" bind:value={name} helper="Lowercase, numbers and dashes. Becomes your URL." mono />
</div>
