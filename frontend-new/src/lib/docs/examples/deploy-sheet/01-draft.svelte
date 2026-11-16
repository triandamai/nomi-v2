<!-- Draft: its first deploy, with what it gets. -->
<script lang="ts">
	import DeploySheet from '$lib/ds/projects/DeploySheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
</script>

<Button variant="secondary" label="Deploy budget-keluarga" onclick={() => (open = true)} />
<DeploySheet bind:open name="budget-keluarga" files={3} url="budget-keluarga-7c21.trian.space" data="none yet · add a database later" ondeploy={() => {}} />
