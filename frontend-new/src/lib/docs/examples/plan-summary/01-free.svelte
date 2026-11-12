<!-- On the free plan, a third of the way through October. -->
<script lang="ts">
	import Button from '$lib/ds/actions/Button.svelte';
	import MetaList from '$lib/ds/settings/MetaList.svelte';
	import PlanSummary from '$lib/ds/settings/PlanSummary.svelte';
</script>

<div class="w-full max-w-220">
	<PlanSummary
		label="Your plan"
		tag="Free plan"
		plan="Free"
		line="Rp 0 / month · 1M tokens a month"
		allowance={{ label: 'Allowance', value: '21.8K / 1M tokens' }}
		used={0.022}
		projected={0.032}
		legend={{ used: '2% used', projected: '~32K projected by 31 Oct', resets: 'resets Sun 1 Nov · 11 days' }}
	>
		{#snippet actions()}<Button label="Compare plans" variant="secondary" href="#compare" />{/snippet}
		{#snippet aside()}
			<MetaList label="Billing" rows={[{ label: 'Allowance resets', value: 'Sun 1 Nov' }, { label: 'Billing period', value: '1–31 Oct' }, { label: 'Member since', value: '14 Jul 2026' }]} />
		{/snippet}
	</PlanSummary>
</div>
