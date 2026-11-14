<script lang="ts">
	import FormSection from './FormSection.svelte';
	import TextField from '../inputs/TextField.svelte';
	import Button from '../actions/Button.svelte';

	// For tests and the docs: a section with one field and its save button.
	let { danger = false }: { danger?: boolean } = $props();
</script>

<FormSection title="App domains" description="The domain every app Nomi publishes is addressed under." {danger}>
	<TextField label="Apps base domain" name="apps_base_domain" mono />
	{#snippet footer()}<Button label="Save" variant="inverse" type="submit" />{/snippet}
</FormSection>
