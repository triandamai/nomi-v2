<!-- The live deploy, a failed one, and the first one to roll back to. -->
<script lang="ts">
	import DeployRow from '$lib/ds/projects/DeployRow.svelte';

	const labels = { live: 'Live', failed: 'Failed', running: 'Publishing', rollback: 'Roll back to {number}' };
</script>

<ul class="flex w-full max-w-170 flex-col">
	<DeployRow number="#3" title="Deploy #3" meta="08:50 today · by Koda" state="failed" error="Missing src/lib/menu.ts" {labels} />
	<DeployRow number="#2" title="Deploy #2" meta="08:42 today · by Mario" state="live" {labels} />
	<DeployRow number="#1" title="First deploy" meta="08:18 today · by Mario" state="rollback" {labels} onrollback={() => {}} />
</ul>
