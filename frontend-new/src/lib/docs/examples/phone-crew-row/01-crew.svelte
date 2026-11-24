<!-- The crew: five faces, each opening the crew's settings. -->
<script lang="ts">
	import PhoneCrewRow from '$lib/ds/thread/PhoneCrewRow.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneCrewRow
		label="Your crew"
		href="#"
		members={[
			{ agent: 'nomi', name: 'Nomi' },
			{ agent: 'dana', name: 'Dana' },
			{ agent: 'kala', name: 'Kala' },
			{ agent: 'maya', name: 'Maya' },
			{ agent: 'tara', name: 'Tara' },
		]}
	/>
</div>
