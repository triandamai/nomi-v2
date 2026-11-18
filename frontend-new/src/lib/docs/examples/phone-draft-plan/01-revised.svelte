<!-- Revised: V2, the added step marked +. -->
<script lang="ts">
	import PhoneDraftPlan from '$lib/ds/thread/PhoneDraftPlan.svelte';
</script>

<div class="w-89.5">
	<PhoneDraftPlan
		title="1–2 Nov"
		version={2}
		intro="Tell Nomi what to change. Nothing runs until you say so."
		steps={[
			{ label: 'Find Whoosh trains: Sat 1 Nov out, Sun 2 Nov back', owner: { agent: 'nomi', name: 'Nomi' } },
			{ label: 'Rainy-day backup for Sunday: museums and cafés along Braga', owner: { agent: 'nomi', name: 'Nomi' }, added: true },
			{ label: 'Set a Rp 3,5 jt trip budget and record spending', owner: { agent: 'dana', name: 'Dana' } },
		]}
	/>
</div>
