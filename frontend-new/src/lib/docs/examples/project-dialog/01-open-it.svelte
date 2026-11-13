<!-- Open it: a native modal at one of the projects widths. Escape or Cancel closes it. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectDialog from '$lib/ds/projects/ProjectDialog.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Open dialog" onclick={() => (open = true)} />
<ProjectDialog bind:open width={460} title="Import a repo into Trian's space" description="Koda reads it first and tells you what it found.">
	{#snippet actions()}
		<Button variant="secondary" label="Cancel" onclick={() => (open = false)} />
	{/snippet}
</ProjectDialog>
