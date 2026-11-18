<!-- Questions: one at a time, then the review. -->
<script lang="ts">
	import PhoneClarifySheet from '$lib/ds/thread/PhoneClarifySheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
	const questions = [
		{ topic: 'Getting there', title: 'How do you want to get there?', options: [{ title: 'Whoosh high-speed train', detail: '45 min from Halim · Rp 250rb each way', recommended: true }, { title: 'Drive', detail: '3–4 hours with weekend traffic' }, { title: 'Travel shuttle', detail: 'Door to door · ~5 hours' }] },
		{ topic: 'Cover', title: 'What should the plan cover?', multiple: true, options: [{ title: 'Train times' }, { title: 'Where to stay' }, { title: 'Food spots' }] },
	];
</script>

<Button variant="secondary" label="Open the questions" onclick={() => (open = true)} />
<PhoneClarifySheet bind:open chat="Bandung weekend" {questions} onfinish={() => (open = false)} />
