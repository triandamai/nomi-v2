<!-- Editable: reorder, reassign or remove steps; nothing runs until you say so. -->
<script lang="ts">
	import DraftPlanCard from '$lib/ds/chat/DraftPlanCard.svelte';

	let steps = $state([
		{ label: 'Read the rent transfers in your BCA statements', owner: { agent: 'maya' as const, name: 'Maya' } },
		{ label: "Record each month's rent as an expense", owner: { agent: 'dana' as const, name: 'Dana' } },
		{ label: 'Remind you on the 25th, three days before rent is due', owner: { agent: 'kala' as const, name: 'Kala' } },
	]);

	function move(from: number, to: number) {
		const next = [...steps];
		const [step] = next.splice(from, 1);
		next.splice(to, 0, step);
		steps = next;
	}
</script>

<div class="w-full max-w-170">
	<DraftPlanCard
		intro="How I'd set up the kos rent tracker. Reorder, reassign or remove steps; nothing runs until you say so."
		{steps}
		estimate="~2 min · Maya, Dana, Kala"
		onmove={move}
		onremove={(index) => (steps = steps.filter((_, at) => at !== index))}
		onrun={() => {}}
		onchanges={() => {}}
		ondiscard={() => {}}
	/>
</div>
