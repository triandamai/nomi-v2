<!-- A decision: the suggested option first. -->
<script lang="ts">
	import PhoneDecision from '$lib/ds/thread/PhoneDecision.svelte';
</script>

<div class="w-89.5">
	<PhoneDecision question="When should I remind you about rent?" options={[{ title: 'On the 25th' }, { title: '3 days before', detail: 'Rent is due on the 28th', suggested: true }]} onconfirm={() => {}} />
</div>
