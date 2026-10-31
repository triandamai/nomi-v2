<!-- Options: each can be on or off on its own; the words say what ticking does. -->
<script lang="ts">
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
</script>

<div class="flex flex-col items-start gap-3">
	<Checkbox label="Mask amounts" checked />
	<Checkbox label="Include names" />
	<Checkbox label="Include names" disabled />
</div>
