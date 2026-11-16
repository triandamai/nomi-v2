<!-- Sort: one choice, checked. -->
<script lang="ts">
	import { ArrowDownAZ, Calendar, CircleAlert, Clock } from '@lucide/svelte';
	import SheetOption from '$lib/ds/overlays/SheetOption.svelte';

	let sort = $state('updated');
	const options = [
		{ value: 'updated', label: 'Updated', icon: Clock },
		{ value: 'status', label: 'Status · needs you first', icon: CircleAlert },
		{ value: 'name', label: 'Name · A–Z', icon: ArrowDownAZ },
		{ value: 'created', label: 'Created', icon: Calendar },
	];
</script>

<div class="flex w-97.5 max-w-full flex-col rounded-t-sheet bg-surface">
	{#each options as option (option.value)}<SheetOption label={option.label} icon={option.icon} chosen={sort === option.value} onclick={() => (sort = option.value)} />{/each}
</div>
