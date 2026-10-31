<!-- Review: your answers, a default marked with the reason; nothing runs yet. -->
<script lang="ts">
	import ClarifyReview from '$lib/ds/chat/ClarifyReview.svelte';
</script>

<div class="w-full max-w-170">
	<ClarifyReview
		answers={[
			{ topic: 'Getting there', value: 'Whoosh high-speed train' },
			{ topic: 'Budget', value: 'Rp 3,5 jt max, for both of us' },
			{ topic: 'Dates', value: 'Sat 1 – Sun 2 Nov', isDefault: true },
			{ topic: 'What to cover', value: 'Trains, stay, food, budget, share with Sari' },
		]}
		assumption="You skipped Dates, so I picked the next free weekend. Your rent reminder is on the 25th, so this weekend is busy."
		onedit={() => {}}
		onback={() => {}}
		onconfirm={() => {}}
	/>
</div>
