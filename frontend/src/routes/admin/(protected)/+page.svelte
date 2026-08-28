<script lang="ts">
	import Card from '$lib/components/m3/Card.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const cards = $derived([
		{ label: 'Total Users', value: data.stats.total_users },
		{ label: 'Tokens Today', value: data.stats.tokens_today },
		{ label: 'Tokens All-Time', value: data.stats.tokens_all_time },
		{ label: 'Running Agents', value: data.stats.running_agents },
	]);
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Admin dashboard</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Manage app-wide configuration.
</p>

<div class="mt-6 grid grid-cols-2 gap-4 md:grid-cols-4">
	{#each cards as card (card.label)}
		<Card variant="outlined" class="p-4">
			<p class="md-label-medium" style="color: var(--md-sys-color-on-surface-variant)">{card.label}</p>
			<p class="md-title-large mt-1" style="color: var(--md-sys-color-on-surface)">{card.value.toLocaleString()}</p>
		</Card>
	{/each}
</div>
