<!-- Before a delete: what happens next, the action on a danger button. -->
<script lang="ts">
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
</script>

<ConfirmDialog inline title="Delete Pro?" body="Nobody is on it, so it goes for good. People can’t be put on it again." confirmLabel="Delete plan" cancelLabel="Cancel" onconfirm={() => {}} />
