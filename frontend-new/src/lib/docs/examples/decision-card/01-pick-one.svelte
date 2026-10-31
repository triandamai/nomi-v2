<!-- Pick one: tradeoffs written out, the suggestion picked first. -->
<script lang="ts">
	import DecisionCard from '$lib/ds/chat/DecisionCard.svelte';
</script>

<div class="w-full max-w-170">
	<DecisionCard
		question="When should Kala remind you about rent?"
		options={[
			{ title: '3 days before', detail: '25th, 09:00. Time to top up BCA if needed.', suggested: true },
			{ title: 'On the day', detail: '28th, 07:00. Least noise, no buffer.' },
			{ title: 'Both', detail: '25th and 28th. Two pings a month.' },
		]}
		onconfirm={() => {}}
	/>
</div>
