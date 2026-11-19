<!-- Deployments: the live one, and one to roll back to. -->
<script lang="ts">
	import PhoneList from '$lib/ds/projects/PhoneList.svelte';
	import PhoneListRow from '$lib/ds/projects/PhoneListRow.svelte';
	import Status from '$lib/ds/status/Status.svelte';
</script>

<div class="w-89.5 max-w-full">
	<PhoneList label="Deployments">
		<PhoneListRow lead="#2" title="Deploy #2" meta="Today 08:42 · by Mario">
			{#snippet end()}<Status kind="live" label="Live" />{/snippet}
		</PhoneListRow>
		<PhoneListRow lead="#1" title="First deploy" meta="Today 08:18 · by Mario" />
	</PhoneList>
</div>
