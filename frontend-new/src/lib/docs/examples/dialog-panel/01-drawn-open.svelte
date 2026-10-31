<!-- Drawn open: the box on the scrim, as the sheets show it. In a screen, use Dialog. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import DialogPanel from '$lib/ds/overlays/DialogPanel.svelte';
</script>

<div class="flex w-full items-center justify-center rounded-md bg-overlay p-8">
	<DialogPanel
		id="docs-dialog-panel"
		title="Delete salah-daily-tracker?"
		description="The live URL stops working and the database is removed. Your code stays in your account for 30 days; export it first if you want a copy."
	>
		{#snippet actions()}
			<Button label="Cancel" variant="secondary" />
			<Button label="Delete project" variant="danger" />
		{/snippet}
	</DialogPanel>
</div>
