<!-- Widths: 420 for a confirm, up to 520 for the invite. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectDialogPanel from '$lib/ds/projects/ProjectDialogPanel.svelte';
</script>

<ProjectDialogPanel id="docs-project-dialog-panel" width={420} title="Deploy budget-keluarga?" description="Koda builds the 3 files and puts them live.">
	{#snippet actions()}
		<Button variant="secondary" label="Cancel" />
		<Button variant="inverse" label="Deploy" />
	{/snippet}
</ProjectDialogPanel>
