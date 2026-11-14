<!-- A plan, with a helper. -->
<script lang="ts">
	import Select from '$lib/ds/admin/Select.svelte';

	let plan = $state('pro');
</script>

<div class="w-full max-w-80">
	<Select
		label="Plan"
		bind:value={plan}
		helper="Hidden plans are listed too."
		options={[
			{ value: 'free', label: 'Free · 1M' },
			{ value: 'pro', label: 'Pro · 5M' },
			{ value: 'team', label: 'Team · 20M' },
		]}
	/>
</div>
