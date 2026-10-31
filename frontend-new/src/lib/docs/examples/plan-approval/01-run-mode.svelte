<!-- Run mode: the recommendation picked first, or keep planning with what you want changed. -->
<script lang="ts">
	import PlanApproval from '$lib/ds/chat/PlanApproval.svelte';
</script>

<div class="w-full max-w-170">
	<PlanApproval
		options={[
			{ title: 'Run it, ask before anything leaves Nomi', detail: 'Research runs freely. Tara asks before sharing the doc; Dana only records.', recommended: true },
			{ title: 'Research only', detail: 'Find trains, stays and food. Nothing is drafted, shared or tracked.' },
		]}
		onrun={() => {}}
	/>
</div>
