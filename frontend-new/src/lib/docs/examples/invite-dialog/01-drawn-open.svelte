<!-- Drawn open: type addresses (Enter or a comma adds each), pick a role, send. -->
<script lang="ts">
	import InviteDialog from '$lib/ds/projects/InviteDialog.svelte';
</script>

<InviteDialog inline workspace="Trian's space" emails={['siti@kopikita.id', 'budi.s@gmail.com']} oninvite={() => {}} oncopylink={() => {}} />
