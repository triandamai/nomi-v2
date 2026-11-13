<!-- The next 7 days, and nothing coming up. -->
<script lang="ts">
	import Agenda from '$lib/ds/inbox/Agenda.svelte';
</script>

<div class="flex w-full max-w-75 flex-col gap-6">
	<Agenda
		label="Next 7 days"
		count={4}
		empty="Nothing in the next 7 days."
		days={[
			{ label: 'Today', today: true, items: [{ agent: 'kala', title: 'Stretch', time: '16:00' }, { agent: 'kala', title: 'Call Siti', time: '20:00' }] },
			{ label: 'Thu 22', items: [{ agent: 'dana', title: 'Netflix renews · Rp 186.000', time: '09:00' }] },
			{ label: 'Sun 25', items: [{ agent: 'kala', title: 'Pay PLN electricity', time: '10:00' }] },
		]}
	/>
	<Agenda label="Next 7 days" count={0} empty="Nothing in the next 7 days." days={[]} />
</div>
