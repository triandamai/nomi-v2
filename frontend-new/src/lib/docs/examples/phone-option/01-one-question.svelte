<!-- One question: each answer a card; the chosen one takes the accent. -->
<script lang="ts">
	import PhoneOption from '$lib/ds/mobile/PhoneOption.svelte';

	const options = [
		{ title: 'Whoosh high-speed train', detail: '45 min from Halim · about Rp 250rb each way' },
		{ title: 'Drive', detail: '3–4 hours with weekend traffic, tolls and parking around Rp 400rb.' },
		{ title: 'Travel shuttle', detail: 'Door to door from Kuningan, about 3 hours, Rp 150rb each.' },
	];
	let picked = $state(0);
</script>

<div role="radiogroup" aria-label="Getting there" class="flex w-89.5 max-w-full flex-col gap-2">
	{#each options as option, index (option.title)}
		<PhoneOption name="trip" key={index + 1} {...option} checked={picked === index} onselect={() => (picked = index)} />
	{/each}
</div>
