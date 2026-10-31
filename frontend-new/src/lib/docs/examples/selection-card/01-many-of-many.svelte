<!-- Many of many: a running count and total of what you picked. -->
<script lang="ts">
	import SelectionCard from '$lib/ds/chat/SelectionCard.svelte';

	const items = [
		{ name: 'Netflix Premium', detail: 'monthly · BCA debit', amount: '186.000', monthly: 186_000 },
		{ name: 'Spotify Duo', detail: 'monthly · GoPay', amount: '54.990', monthly: 54_990 },
		{ name: 'iCloud+ 200 GB', detail: 'monthly · Visa •• 4412', amount: '45.000', monthly: 45_000 },
		{ name: 'Fitness First', detail: 'monthly · since Mar', amount: '650.000', monthly: 650_000 },
	];
	let picked = $state([true, true, true, false]);
	const count = $derived(picked.filter(Boolean).length);
	const total = $derived(items.reduce((sum, item, index) => sum + (picked[index] ? item.monthly : 0), 0));
</script>

<div class="w-full max-w-170">
	<SelectionCard
		agent="dana"
		question="Which of these should Dana track as subscriptions?"
		{items}
		selected={picked}
		total="Rp {new Intl.NumberFormat('id-ID').format(total)} / month"
		confirmLabel="Track {count}"
		onchange={(next) => (picked = next)}
		onconfirm={() => {}}
		onskip={() => {}}
	/>
</div>
