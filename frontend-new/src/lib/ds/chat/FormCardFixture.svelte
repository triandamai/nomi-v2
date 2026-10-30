<script lang="ts">
	import FormCard from './FormCard.svelte';
	import TextField from '../inputs/TextField.svelte';

	// A FormCard with fields, for its test.
	let { onsubmit, oncancel }: { onsubmit: () => void; oncancel: () => void } = $props();
</script>

<FormCard agent="kala" title="New reminder" meta="prefilled from chat" submitLabel="Save reminder" {onsubmit} {oncancel}>
	<TextField label="What" value="Pay kos rent to Bu Sri" />
</FormCard>
