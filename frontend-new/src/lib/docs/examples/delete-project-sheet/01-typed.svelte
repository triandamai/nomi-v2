<!-- Delete: export first, then the name typed to confirm. -->
<script lang="ts">
	import DeleteProjectSheet from '$lib/ds/projects/DeleteProjectSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="danger" label="Delete salah-tracker" onclick={() => (open = true)} />
<DeleteProjectSheet bind:open name="salah-tracker" onexport={() => {}} ondelete={() => (open = false)} />
