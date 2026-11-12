<!-- The plan they're on, and the one above it. -->
<script lang="ts">
	import PlanOption from '$lib/ds/settings/PlanOption.svelte';
</script>

<div class="grid w-full max-w-220 gap-4 md:grid-cols-2">
	<PlanOption name="Free" tag={{ label: 'Current', kind: 'go' }} price="Rp 0" per="/ month" description="Everything Nomi does, with a monthly allowance." features={['1M tokens a month', 'The whole crew', 'Projects with a live URL']} />
	<PlanOption name="Pro" price="Rp 149.000" per="/ month" description="For people who hand the crew real work every day." features={['10M tokens a month', 'Everything in Free']} raised />
</div>
