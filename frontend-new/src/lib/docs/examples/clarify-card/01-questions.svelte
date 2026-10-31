<!-- Questions: one at a time, the recommendation picked; number keys pick, Enter goes on. -->
<script lang="ts">
	import ClarifyCard from '$lib/ds/chat/ClarifyCard.svelte';
	import type { ClarifyQuestion } from '$lib/ds/chat/clarify';

	const questions: ClarifyQuestion[] = [
		{
			topic: 'Getting there',
			title: 'How do you want to get to Bandung?',
			helper: 'You and Sari, from Jakarta. This decides the timing of everything else.',
			options: [
				{ title: 'Whoosh high-speed train', detail: '45 min from Halim, about Rp 250rb each way per person. Fits a one-night trip.', recommended: true },
				{ title: 'Drive', detail: '3–4 hours with weekend traffic, tolls and parking around Rp 400rb.' },
				{ title: 'Travel shuttle', detail: 'Door to door from Kuningan, about 3 hours, Rp 150rb each.' },
			],
			other: true,
		},
		{
			topic: 'What to cover',
			title: 'What should the plan cover?',
			helper: 'Pick any. The crew member who owns each part is noted.',
			multiple: true,
			options: [
				{ title: 'Train tickets', detail: 'Nomi finds times; you book on Access by KAI' },
				{ title: 'Trip budget', detail: 'Dana tracks every expense against Rp 3,5 jt' },
				{ title: 'Reminders', detail: 'Kala: pack Friday night, leave Saturday 06:30' },
			],
		},
	];
</script>

<div class="w-full max-w-170">
	<ClarifyCard agent="nomi" title="Before I plan: 2 quick questions" {questions} onfinish={() => {}} onskipall={() => {}} />
</div>
