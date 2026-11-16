<!-- Stop: the question, export first, stop, keep building. -->
<script lang="ts">
	import ActionSheet from '$lib/ds/overlays/ActionSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Stop the build" onclick={() => (open = true)} />
<ActionSheet
	bind:open
	title="Stop building warung-kopi-menu?"
	body="Koda finishes the file it’s writing, then stops. Nothing is deployed."
	actions={[
		{ label: 'Export first', onclick: () => {} },
		{ label: 'Stop building', danger: true, onclick: () => {} },
	]}
	cancel="Keep building"
/>
