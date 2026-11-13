<!-- A configuration section: its fields and its own save. -->
<script lang="ts">
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="w-full max-w-160">
	<FormSection title="App domains" description="The domain every app Nomi publishes is addressed under: each app gets its own name under it.">
		<TextField label="Apps base domain" mono placeholder="apps.example.com" helper="Without https://. Empty: Shipyard’s own apps domain is used." />
		{#snippet footer()}<Button label="Save" variant="inverse" />{/snippet}
	</FormSection>
</div>
