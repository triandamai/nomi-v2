<!-- Two memories about people: one steady, one the crew should check. -->
<script lang="ts">
	import PhoneMemoryCard from '$lib/ds/settings/PhoneMemoryCard.svelte';
</script>

<div class="flex w-97.5 max-w-full flex-col gap-2.5 px-4">
	<PhoneMemoryCard content="Rina is Siti’s sister and runs the till on weekends." strength={{ level: 3, name: 'steady' }} meta="used 4× · learned by Nomi" more="More for this memory" onmore={() => {}} />
	<PhoneMemoryCard
		content="Pak Budi supplies the coffee beans; orders go out on Mondays."
		strength={{ level: 1, name: 'faint' }}
		meta="not used yet · learned by Dana"
		check="Is this still right?"
		more="More for this memory"
		onmore={() => {}}
	/>
</div>
