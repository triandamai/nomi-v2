<!-- Error and disabled: the error says how to fix it; a disabled field says why. -->
<script lang="ts">
	import TextField from '$lib/ds/inputs/TextField.svelte';
</script>

<div class="grid w-full max-w-170 gap-6 sm:grid-cols-2">
	<TextField label="Project name" value="Budget Keluarga!" error="Use lowercase and dashes, e.g. budget-keluarga." mono />
	<TextField label="Project name" value="salah-daily-tracker" helper="Project name can't change after deploy." mono disabled />
</div>
