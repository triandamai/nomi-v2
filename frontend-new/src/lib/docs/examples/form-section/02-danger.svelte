<!-- Danger: a section whose action can't be undone. -->
<script lang="ts">
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="w-full max-w-160">
	<FormSection danger title="Terminate app" description="Removes the app everywhere: every environment, its data, the dev database and secrets.">
		{#snippet footer()}<Button label="Terminate app" variant="danger" />{/snippet}
	</FormSection>
</div>
