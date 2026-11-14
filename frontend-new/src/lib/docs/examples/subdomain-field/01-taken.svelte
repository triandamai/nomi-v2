<!-- A name that’s taken, with free ones to pick. -->
<script lang="ts">
	import SubdomainField from '$lib/ds/projects/SubdomainField.svelte';
</script>

<div class="w-full max-w-170">
	<SubdomainField
		value="kopi"
		suffix="trian.space"
		check={{ state: 'taken', message: 'That name is taken.', suggestions: ['kopi-1', 'kopi-bu-siti'] }}
		labels={{ field: 'Address', checking: 'Checking…', available: 'Available', suggestions: 'Free instead' }}
	/>
</div>
