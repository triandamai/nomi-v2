<!-- Running: each step's real state, what it produced or waits for, and who owns it. -->
<script lang="ts">
	import PlanCard from '$lib/ds/chat/PlanCard.svelte';
</script>

<div class="w-full max-w-170">
	<PlanCard
		subject="Weekly money check"
		onstop={() => {}}
		steps={[
			{ label: 'Pull October transactions', meta: '142 rows', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
			{ label: 'Find the categories that grew', meta: 'food +48% vs September', state: 'done', owner: { agent: 'dana', name: 'Dana' } },
			{ label: 'Draft a weekly food budget', meta: 'running · 00:06', state: 'running', owner: { agent: 'nomi', name: 'Nomi' } },
			{ label: 'Set a Sunday check-in reminder', meta: 'waits for step 3', state: 'waiting', owner: { agent: 'kala', name: 'Kala' } },
		]}
	/>
</div>
